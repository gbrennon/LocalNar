#![allow(dead_code)]
use application::{
    errors::ModelDownloadError,
    ports::outbound::{DownloadProgress, DownloadProgressPort, ModelDownloaderPort},
};
use domain::{ModelArtifact, RemoteModelFile};

use crate::common::fakes::model_fixture::ModelFixture;

/// A downloader that always stages the fixture bytes successfully.
pub struct FakeStagingDownloader;

impl ModelDownloaderPort for FakeStagingDownloader {
    async fn fetch(
        &self,
        remote: &RemoteModelFile,
        progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, ModelDownloadError> {
        let total = remote.size();

        progress.report(DownloadProgress::Started { total });
        progress.report(DownloadProgress::Advanced {
            transferred: total,
            total,
        });
        progress.report(DownloadProgress::Finished);

        Ok(ModelFixture::artifact())
    }
}
