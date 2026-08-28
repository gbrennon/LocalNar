use crate::byte_length::ByteLength;
use crate::domain_error::DomainError;
use crate::model_file_name::ModelFileName;
use crate::model_repository::ModelRepository;
use crate::sha256::Sha256;

/// Everything an upstream registry discloses about one downloadable file.
///
/// The `sha256` is optional because not every registry advertises a digest; a
/// missing digest simply means that integrity cannot be proven by checksum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteModelFile {
    repository: ModelRepository,
    file: ModelFileName,
    size: ByteLength,
    sha256: Option<Sha256>,
}

impl RemoteModelFile {
    /// Builds a disclosed remote file with its announced metadata.
    pub fn new(
        repository: ModelRepository,
        file: ModelFileName,
        size: ByteLength,
        sha256: Option<Sha256>,
    ) -> Self {
        Self {
            repository,
            file,
            size,
            sha256,
        }
    }

    /// The repository the file lives in.
    pub fn repository(&self) -> &ModelRepository {
        &self.repository
    }

    /// The plain name of the file inside the repository.
    pub fn file(&self) -> &ModelFileName {
        &self.file
    }

    /// The announced byte length of the file.
    pub fn size(&self) -> ByteLength {
        self.size
    }

    /// Whether the registry disclosed a digest to verify against.
    pub fn has_checksum(&self) -> bool {
        self.sha256.is_some()
    }

    /// Verifies an actual digest against the advertised one.
    ///
    /// Verifying against a file without an advertised digest always succeeds;
    /// a mismatch is reported as `DomainError::IntegrityMismatch`.
    pub fn verify_against(&self, actual: Sha256) -> Result<(), DomainError> {
        match self.sha256 {
            None => Ok(()),
            Some(expected) if expected == actual => Ok(()),
            Some(expected) => Err(DomainError::IntegrityMismatch {
                expected: expected.to_hex(),
                actual: actual.to_hex(),
            }),
        }
    }
}

#[cfg(test)]
mod remote_model_file_tests {
    use crate::byte_length::ByteLength;
    use crate::domain_error::DomainError;
    use crate::model_file_name::ModelFileName;
    use crate::model_repository::ModelRepository;
    use crate::model_repository_id::ModelRepositoryId;
    use crate::remote_model_file::RemoteModelFile;
    use crate::sha256::Sha256;

    const DIGEST_HEX: &str = "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3dfa96d1b0f6a55a0f9f0f7e8";

    fn remote_with(sha256: Option<Sha256>) -> RemoteModelFile {
        let identifier = ModelRepositoryId::parse("org/name").expect("valid id");
        let repository = ModelRepository::at_default_revision(identifier);
        let file = ModelFileName::new("model.gguf").expect("valid file");
        RemoteModelFile::new(repository, file, ByteLength::new(1024), sha256)
    }

    #[test]
    fn verification_passes_when_no_checksum_is_advertised() {
        let checksum = Sha256::parse(DIGEST_HEX).expect("valid digest");
        assert!(remote_with(None).verify_against(checksum).is_ok());
    }

    #[test]
    fn verification_passes_when_the_digest_matches() {
        let checksum = Sha256::parse(DIGEST_HEX).expect("valid digest");
        assert!(remote_with(Some(checksum)).verify_against(checksum).is_ok());
    }

    #[test]
    fn verification_fails_when_the_digest_differs() {
        let expected = Sha256::parse(DIGEST_HEX).expect("valid digest");
        let actual = Sha256::from_bytes([1u8; 32]);
        assert!(matches!(
            remote_with(Some(expected)).verify_against(actual),
            Err(DomainError::IntegrityMismatch { .. })
        ));
    }

    #[test]
    fn accessors_expose_repository_file_and_size() {
        let file = remote_with(None);
        assert_eq!(file.file().as_str(), "model.gguf");
        assert_eq!(file.size(), ByteLength::new(1024));
        assert_eq!(file.repository().to_string(), "org/name@main");
        assert!(!file.has_checksum());
        let signed = remote_with(Some(Sha256::parse(DIGEST_HEX).expect("valid digest")));
        assert!(signed.has_checksum());
    }
}
