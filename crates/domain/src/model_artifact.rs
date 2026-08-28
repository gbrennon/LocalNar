use std::path::{Path, PathBuf};

use crate::byte_length::ByteLength;
use crate::model_repository::ModelRepository;

/// A byte stream that a downloader has produced but that has not yet been
/// committed to the durable model library.
///
/// The `staged_at` path is owned by the downloader/operator; the model only
/// records where the bytes sit and how large they are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelArtifact {
    staged_at: PathBuf,
    size: ByteLength,
    origin: ModelRepository,
}

impl ModelArtifact {
    /// Wraps a staged file produced for a given upstream origin.
    pub fn new(staged_at: impl Into<PathBuf>, size: ByteLength, origin: ModelRepository) -> Self {
        Self {
            staged_at: staged_at.into(),
            size,
            origin,
        }
    }

    /// The path where the staged bytes currently live.
    pub fn staged_at(&self) -> &Path {
        &self.staged_at
    }

    /// The number of bytes staged.
    pub fn size(&self) -> ByteLength {
        self.size
    }

    /// The upstream repository the bytes were drawn from.
    pub fn origin(&self) -> &ModelRepository {
        &self.origin
    }
}
