use localnar_domain::ModelInventory;

use crate::errors::library_error::LibraryError;

/// Outbound contract for reading everything the durable library holds.
///
/// `ModelLibraryPort` answers about one model the caller already names; this
/// port answers about models nobody has named yet, which is what managing a
/// machine's library requires. The two are separate roles: installing needs to
/// look one model up, managing needs to see the whole store.
///
/// The answer is a snapshot taken without side effects. Reading it must stay
/// cheap enough to serve an interactive listing, so an implementation reports
/// the state it has already recorded and never re-reads a replica's bytes: an
/// entry therefore comes back `Downloaded` or `Verified`, and proving or
/// disproving that reading is the verification use case's work.
///
/// Whatever the library keeps that stands for no model is left out rather than
/// described as an unusable entry.
pub trait ModelInventoryPort: Send + Sync {
    /// Describes the library's location and every replica it holds.
    async fn enumerate(&self) -> Result<ModelInventory, LibraryError>;
}
