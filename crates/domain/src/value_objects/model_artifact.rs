use std::path::{Path, PathBuf};

use crate::value_objects::ByteLength;

/// A byte stream that a downloader has produced but that has not yet been
/// committed to the durable model library.
///
/// The `staged_at` path is owned by the downloader/operator; the model only
/// records where the bytes sit and how large they are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelArtifact {
    staged_at: PathBuf,
    size: ByteLength,
}

impl ModelArtifact {
    /// Wraps a staged file that a downloader has produced.
    pub fn new(staged_at: impl Into<PathBuf>, size: ByteLength) -> Self {
        Self {
            staged_at: staged_at.into(),
            size,
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
}

#[cfg(test)]
mod model_artifact_tests {
    use std::path::Path;

    use crate::value_objects::{ByteLength, ModelArtifact};

    #[test]
    fn a_staged_artifact_reports_where_its_bytes_sit_and_how_large_they_are() {
        let artifact = ModelArtifact::new("/tmp/staging/qwen.gguf", ByteLength::new(4_096));

        assert_eq!(artifact.staged_at(), Path::new("/tmp/staging/qwen.gguf"));
        assert_eq!(artifact.size(), ByteLength::new(4_096));
    }
}
