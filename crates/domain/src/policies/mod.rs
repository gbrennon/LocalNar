//! Rules that decide something without holding state of their own.
//!
//! A policy takes the values it judges as arguments, so the same inputs
//! always yield the same decision.

mod model_weight_choice;

pub use model_weight_choice::ModelWeightChoice;
