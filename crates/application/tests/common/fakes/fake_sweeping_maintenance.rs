#![allow(dead_code)]
use application::errors::LibraryError;
use application::ports::outbound::LibraryMaintenancePort;
use domain::DiscardedStray;

use crate::common::fakes::model_fixture::ModelFixture;

/// Maintenance of a library carrying leftovers around its replicas.
pub struct FakeSweepingMaintenance;

impl LibraryMaintenancePort for FakeSweepingMaintenance {
    async fn discard_strays(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        Ok(vec![
            ModelFixture::discarded_digest_record(),
            ModelFixture::discarded_companion_digest_record(),
            ModelFixture::discarded_emptied_directory(),
        ])
    }
}
