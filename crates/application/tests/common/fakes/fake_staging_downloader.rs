#![allow(dead_code)]
use application::errors::ModelDownloadError;
use application::ports::outbound::{DownloadProgress, DownloadProgressPort, ModelDownloaderPort};
use domain::{ModelArtifact, RemoteModelFile};

use crate::common::fakes::model_fixture::ModelFixture;

/// A downloader that always stages the fixture bytes successfully.
pub struct FakeStagingDownloader;

impl ModelDownloaderPort for FakeStagingDownloader {
    async fn fetch<Progress>(
        &self,
        remote: &RemoteModelFile,
        progress: &Progress,
    ) -> Result<ModelArtifact, ModelDownloadError>
    where
        Progress: DownloadProgressPort,
    {
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
