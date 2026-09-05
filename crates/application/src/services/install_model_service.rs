use std::ops::ControlFlow;

use localnar_domain::{Checksum, InstalledModel, ModelSpec, ModelState};

use crate::{
    errors::install_model_error::InstallModelError,
    ports::{
        inbound::InstallModelPort,
        outbound::{
            download_progress_port::DownloadProgressPort,
            model_downloader_port::ModelDownloaderPort, model_library_port::ModelLibraryPort,
            remote_model_registry_port::RemoteModelRegistryPort,
        },
    },
};

/// The use case that brings one model into its installed, verified state.
///
/// It owns no I/O: every external effect is delegated to an injected port, so
/// the install decisions stay testable against fakes. The ports are type
/// parameters rather than trait objects, which keeps the calls statically
/// dispatched and allocation free; an outer layer that needs to choose an
/// adapter at runtime erases the type at its own boundary.
pub struct InstallModelService<Registry, Downloader, Library, Progress>
where
    Registry: RemoteModelRegistryPort,
    Downloader: ModelDownloaderPort,
    Library: ModelLibraryPort,
    Progress: DownloadProgressPort,
{
    registry: Registry,
    downloader: Downloader,
    library: Library,
    progress: Progress,
}

impl<Registry, Downloader, Library, Progress>
    InstallModelService<Registry, Downloader, Library, Progress>
where
    Registry: RemoteModelRegistryPort,
    Downloader: ModelDownloaderPort,
    Library: ModelLibraryPort,
    Progress: DownloadProgressPort,
{
    /// Compose the use case from the outbound ports.
    pub fn new(
        registry: Registry,
        downloader: Downloader,
        library: Library,
        progress: Progress,
    ) -> Self {
        Self {
            registry,
            downloader,
            library,
            progress,
        }
    }

    async fn fetch_and_commit(&self, spec: &ModelSpec) -> Result<ModelState, InstallModelError> {
        let remote = self
            .registry
            .resolve_model_file(spec.repository(), spec.file())
            .await?;
        let artifact = self.downloader.fetch(&remote, &self.progress).await?;

        Ok(self.library.commit_artifact(spec, &artifact).await?)
    }

    async fn verify(&self, spec: &ModelSpec) -> Result<ModelState, InstallModelError> {
        let remote = self
            .registry
            .resolve_model_file(spec.repository(), spec.file())
            .await?;

        Ok(self
            .library
            .verify_integrity(spec, remote.checksum())
            .await?)
    }

    async fn repair(&self, spec: &ModelSpec) -> Result<ModelState, InstallModelError> {
        self.fetch_and_commit(spec).await
    }

    /// Settle on the installed replica for `spec`.
    async fn settle(
        &self,
        spec: &ModelSpec,
    ) -> Result<ControlFlow<InstalledModel, ModelState>, InstallModelError> {
        Ok(ControlFlow::Break(self.library.locate(spec).await?))
    }

    /// Fetch the missing bytes once; a repeated attempt means upstream never
    /// delivered them.
    async fn advance_missing(
        &self,
        spec: &ModelSpec,
        fetched: &mut bool,
    ) -> Result<ControlFlow<InstalledModel, ModelState>, InstallModelError> {
        if *fetched {
            return Err(InstallModelError::UpstreamUnavailable);
        }
        *fetched = true;
        Ok(ControlFlow::Continue(self.fetch_and_commit(spec).await?))
    }

    /// Verify freshly downloaded bytes once; if verification already ran, the
    /// replica is treated as installed.
    async fn advance_downloaded(
        &self,
        spec: &ModelSpec,
        verified: &mut bool,
    ) -> Result<ControlFlow<InstalledModel, ModelState>, InstallModelError> {
        if *verified {
            return self.settle(spec).await;
        }
        *verified = true;
        Ok(ControlFlow::Continue(self.verify(spec).await?))
    }

    /// Repair a checksum mismatch once; a second mismatch is unresolvable.
    async fn advance_mismatch(
        &self,
        spec: &ModelSpec,
        expected: &Checksum,
        actual: &Checksum,
        verified: &mut bool,
        repaired: &mut bool,
    ) -> Result<ControlFlow<InstalledModel, ModelState>, InstallModelError> {
        if *repaired {
            return Err(InstallModelError::UnresolvedIntegrity {
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
        *repaired = true;
        *verified = false;
        Ok(ControlFlow::Continue(self.repair(spec).await?))
    }

    /// Takes a single step from `state`, either settling on an installed
    /// replica or reporting the next state to drive towards.
    ///
    /// Each attempt flag guards its transition so the driving loop performs at
    /// most one fetch, one verification, and one repair; a guard that has
    /// already fired turns an otherwise repeatable transition into a terminal
    /// error or location.
    async fn advance(
        &self,
        spec: &ModelSpec,
        state: ModelState,
        fetched: &mut bool,
        verified: &mut bool,
        repaired: &mut bool,
    ) -> Result<ControlFlow<InstalledModel, ModelState>, InstallModelError> {
        match state {
            ModelState::Verified => self.settle(spec).await,
            ModelState::Missing => self.advance_missing(spec, fetched).await,
            ModelState::Downloaded => self.advance_downloaded(spec, verified).await,
            ModelState::IntegrityMismatch { expected, actual } => {
                self.advance_mismatch(spec, &expected, &actual, verified, repaired)
                    .await
            }
        }
    }
}

impl<Registry, Downloader, Library, Progress> InstallModelPort
    for InstallModelService<Registry, Downloader, Library, Progress>
where
    Registry: RemoteModelRegistryPort,
    Downloader: ModelDownloaderPort,
    Library: ModelLibraryPort,
    Progress: DownloadProgressPort,
{
    /// Drives `spec` towards the verified state, performing at most one fetch,
    /// one verification, and one repair attempt.
    ///
    /// Settling on `Verified` or on a `Downloaded` replica that has already
    /// been through verification both mean the bytes are installed, so both
    /// answer with the replica's location; the two are told apart by whether
    /// upstream advertised a checksum to prove it against.
    async fn execute(&self, spec: &ModelSpec) -> Result<InstalledModel, InstallModelError> {
        let mut state = self.library.installed_state(spec).await?;
        let mut fetched = false;
        let mut verified = false;
        let mut repaired = false;

        loop {
            match self
                .advance(spec, state, &mut fetched, &mut verified, &mut repaired)
                .await?
            {
                ControlFlow::Break(installed) => return Ok(installed),
                ControlFlow::Continue(next) => state = next,
            }
        }
    }
}
