//! Things identified by who they are rather than by what they hold.
//!
//! An entity keeps its identity across changes to its other attributes.

mod discarded_stray;
mod installed_model;

mod managed_model;
mod model_inventory;
mod removed_model;

pub use discarded_stray::DiscardedStray;
pub use installed_model::InstalledModel;
pub use managed_model::ManagedModel;
pub use model_inventory::ModelInventory;
pub use removed_model::RemovedModel;
