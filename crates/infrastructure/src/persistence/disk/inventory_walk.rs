//! Reading the replicas held by a library's directory hierarchy.

use std::path::{Path, PathBuf};

use application::errors::LibraryError;
use domain::{
    ByteLength, InstalledModel, ManagedModel, ModelFileName, ModelRepository, ModelRepositoryId,
    ModelRevision, ModelSpec, ModelState,
};

use super::library_tree::LibraryTree;
use super::model_library::DiskModelLibrary;

/// Reads every replica the `<owner>/<name>/<revision>/<file>` hierarchy under a
/// root holds.
///
/// The walk reports only what the hierarchy already records and never reads a
/// replica's bytes, so a library of large models costs a directory walk to read
/// rather than a hash of everything in it.
pub(super) struct InventoryWalk<'root> {
    root: &'root Path,
}

impl<'root> InventoryWalk<'root> {
    /// Prepares to walk the hierarchy beneath `root`.
    pub(super) fn rooted_at(root: &'root Path) -> Self {
        Self { root }
    }

    /// Describes every replica the hierarchy holds, ordered by the model each
    /// one is named for.
    ///
    /// Whatever the hierarchy holds that names no model is left out instead of
    /// described as an unusable entry: a digest note, an entry lying at the
    /// wrong depth, and a path segment the domain refuses all stand for no
    /// model. The filesystem answers a directory in whatever order suits it, so
    /// the replicas are ordered by name and two walks of an unchanged library
    /// read alike.
    pub(super) async fn replicas(&self) -> Result<Vec<ManagedModel>, LibraryError> {
        let mut replicas = Vec::new();

        for (owner, owner_path) in Self::child_directories(self.root).await? {
            for (name, name_path) in Self::child_directories(&owner_path).await? {
                for (revision, revision_path) in Self::child_directories(&name_path).await? {
                    let repository = format!("{owner}/{name}");
                    replicas.extend(
                        Self::replicas_within(&repository, &revision, &revision_path).await?,
                    );
                }
            }
        }

        replicas.sort_by_cached_key(|replica| replica.spec().to_string());

        Ok(replicas)
    }

    /// Lists the directories `parent` holds, each paired with the path segment
    /// it contributes to a model's name.
    ///
    /// Only a directory can carry the next segment of the hierarchy, and only a
    /// segment that reads as text can appear in a model's name, so anything
    /// else stands for no model and its branch is left unwalked.
    async fn child_directories(parent: &Path) -> Result<Vec<(String, PathBuf)>, LibraryError> {
        let mut children = Vec::new();

        for entry in LibraryTree::entries_of(parent).await? {
            let path = entry.path();
            let Some(described) = LibraryTree::describe(&path).await? else {
                continue;
            };

            if !described.is_dir() {
                continue;
            }

            let Some(segment) = path.file_name().and_then(|segment| segment.to_str()) else {
                continue;
            };

            children.push((segment.to_owned(), path));
        }

        Ok(children)
    }

    /// Describes every replica the revision directory at `directory` holds.
    ///
    /// A digest note sits beside the replica it describes rather than standing
    /// for a model of its own. The size reported is the one the filesystem
    /// holds now, so an install that was interrupted is described at whatever
    /// length it reached, and the state reported is the one the library already
    /// recorded, so a replica it once proved comes back proven and every other
    /// replica comes back merely downloaded.
    async fn replicas_within(
        repository: &str,
        revision: &str,
        directory: &Path,
    ) -> Result<Vec<ManagedModel>, LibraryError> {
        let mut replicas = Vec::new();

        for entry in LibraryTree::entries_of(directory).await? {
            let path = entry.path();
            let Some(described) = LibraryTree::describe(&path).await? else {
                continue;
            };

            if !described.is_file() || DiskModelLibrary::companion_of(&path).is_some() {
                continue;
            }

            let Some(file) = path.file_name().and_then(|file| file.to_str()) else {
                continue;
            };

            let Some(spec) = Self::named_model(repository, revision, file) else {
                continue;
            };

            let digest =
                DiskModelLibrary::recorded_digest(&DiskModelLibrary::sidecar_of(&path)).await;
            let state = match digest {
                Some(_) => ModelState::Verified,
                None => ModelState::Downloaded,
            };
            let size = ByteLength::new(described.len());

            replicas.push(ManagedModel::new(
                InstalledModel::new(spec, path, size, digest),
                state,
            ));
        }

        Ok(replicas)
    }

    /// Names the model the given path segments stand for, or nothing when they
    /// stand for none.
    ///
    /// The library writes each model at a path built from its name, so the name
    /// is read back out of the path. Segments the domain refuses never came
    /// from a name it accepted, so whatever left them there was not an install.
    fn named_model(repository: &str, revision: &str, file: &str) -> Option<ModelSpec> {
        let identifier = ModelRepositoryId::parse(repository).ok()?;
        let revision = ModelRevision::new(revision).ok()?;
        let file = ModelFileName::new(file).ok()?;

        Some(ModelSpec::new(
            ModelRepository::new(identifier, revision),
            file,
            vec![],
        ))
    }
}
