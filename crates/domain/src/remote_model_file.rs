use crate::byte_length::ByteLength;
use crate::checksum::Checksum;
use crate::model_file_name::ModelFileName;
use crate::model_repository::ModelRepository;
use crate::model_spec::ModelSpec;

/// One file offered by a remote repository.
///
/// The size and optional digest are the only pieces the domain needs for
/// automation; all other metadata (e.g. `likes`, `tags`) lives in the
/// infrastructure adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteModelFile {
    repository: ModelRepository,
    file: ModelFileName,
    size: ByteLength,
    checksum: Option<Checksum>,
}

impl RemoteModelFile {
    pub fn new(
        repository: ModelRepository,
        file: ModelFileName,
        size: ByteLength,
        checksum: Option<Checksum>,
    ) -> Self {
        Self {
            repository,
            file,
            size,
            checksum,
        }
    }

    pub fn repository(&self) -> &ModelRepository {
        &self.repository
    }

    pub fn file(&self) -> &ModelFileName {
        &self.file
    }

    pub fn size(&self) -> ByteLength {
        self.size
    }

    /// The install intent that downloading this file would satisfy.
    ///
    /// A search result is therefore directly actionable: the operator picks a
    /// row and the intent follows from it, with nothing further to supply.
    pub fn to_spec(&self) -> ModelSpec {
        ModelSpec::new(self.repository.clone(), self.file.clone())
    }

    /// The digest the registry advertised, when it disclosed one.
    pub fn checksum(&self) -> Option<Checksum> {
        self.checksum
    }
}
