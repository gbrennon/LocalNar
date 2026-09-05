//! Reading the replicas held by a library's directory hierarchy.

use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};

use localnar_application::errors::LibraryError;
use localnar_domain::{
    ByteLength, Checksum, InstalledModel, ManagedModel, ModelFileName, ModelRepository,
    ModelRepositoryId, ModelRevision, ModelSpec, ModelState,
};

use super::{library_tree::LibraryTree, model_library::DiskModelLibrary};

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
            replicas.extend(Self::replicas_under_owner(&owner, &owner_path).await?);
        }

        replicas.sort_by_cached_key(|replica| replica.spec().to_string());

        Ok(replicas)
    }

    /// Describes every replica held beneath one owner directory, across all of
    /// its model-name directories.
    ///
    /// Only directories carry the next segment of the hierarchy, so a name that
    /// reads as text is walked further and anything else stands for no model.
    async fn replicas_under_owner(
        owner: &str,
        owner_path: &Path,
    ) -> Result<Vec<ManagedModel>, LibraryError> {
        let mut replicas = Vec::new();

        for (name, name_path) in Self::child_directories(owner_path).await? {
            replicas.extend(Self::replicas_under_name(owner, &name, &name_path).await?);
        }

        Ok(replicas)
    }

    /// Describes every replica held beneath one model-name directory, across
    /// all of its revision directories.
    ///
    /// The owner and name together form the repository each replica is named
    /// for, so both are carried down to the revision that finally holds files.
    async fn replicas_under_name(
        owner: &str,
        name: &str,
        name_path: &Path,
    ) -> Result<Vec<ManagedModel>, LibraryError> {
        let mut replicas = Vec::new();

        for (revision, revision_path) in Self::child_directories(name_path).await? {
            let repository = format!("{owner}/{name}");
            replicas.extend(Self::replicas_within(&repository, &revision, &revision_path).await?);
        }

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
            if let Some(replica) = Self::replica_at(repository, revision, entry.path()).await? {
                replicas.push(replica);
            }
        }

        Ok(replicas)
    }

    /// Describes the replica held at `path`, or nothing when `path` holds none.
    async fn replica_at(
        repository: &str,
        revision: &str,
        path: PathBuf,
    ) -> Result<Option<ManagedModel>, LibraryError> {
        let Some(described) = LibraryTree::describe(&path).await? else {
            return Ok(None);
        };

        if !Self::is_replica_file(&described, &path) {
            return Ok(None);
        }

        let Some(spec) = Self::spec_at(&path, repository, revision) else {
            return Ok(None);
        };

        let digest = DiskModelLibrary::recorded_digest(&DiskModelLibrary::sidecar_of(&path)).await;
        let state = Self::state_of(&digest);
        let size = ByteLength::new(described.len());

        Ok(Some(ManagedModel::new(
            InstalledModel::new(spec, path, size, digest),
            state,
        )))
    }

    /// Answers whether `path` is a replica rather than a digest note or a
    /// directory.
    fn is_replica_file(described: &Metadata, path: &Path) -> bool {
        described.is_file() && DiskModelLibrary::companion_of(path).is_none()
    }

    /// Names the model held at `path`, when its segments name one.
    fn spec_at(path: &Path, repository: &str, revision: &str) -> Option<ModelSpec> {
        let file = path.file_name().and_then(|file| file.to_str())?;
        Self::named_model(repository, revision, file)
    }

    /// The state a replica is read back in, given whether a digest was recorded
    /// for it.
    fn state_of(digest: &Option<Checksum>) -> ModelState {
        match digest {
            Some(_) => ModelState::Verified,
            None => ModelState::Downloaded,
        }
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
