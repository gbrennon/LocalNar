//! Discarding one replica from the filesystem-backed library.

use application::{errors::LibraryError, ports::outbound::ModelEvictionPort};
use domain::{ModelSpec, RemovedModel};

use super::{library_tree::LibraryTree, model_library::DiskModelLibrary};

impl ModelEvictionPort for DiskModelLibrary {
    /// Discards the replica of `model`, along with everything recorded for it.
    ///
    /// The size is read before anything goes, because afterwards there is
    /// nothing left to measure. Leaving the digest note behind would leave the
    /// library claiming to have proved bytes it no longer holds, so the note
    /// goes with the bytes and a later reading reports the model as missing
    /// rather than as an unproven replica. The directories the model alone
    /// needed go with it too, so the library does not fill up with repositories
    /// holding nothing.
    ///
    /// A model the library never held costs nothing to discard and reclaims
    /// nothing, which is the honest answer to give a caller that then decides
    /// for itself whether the operator asked for something absent.
    async fn evict(&self, model: &ModelSpec) -> Result<RemovedModel, LibraryError> {
        let path = self.model_file_path(model);
        let reclaimed = LibraryTree::occupied_space(&path).await?;

        LibraryTree::discard_file(&path).await?;
        LibraryTree::discard_file(&self.checksum_file_path(model)).await?;
        LibraryTree::discard_emptied_ancestors(self.root(), &path).await?;

        Ok(RemovedModel::new(model.clone(), path, reclaimed))
    }
}
