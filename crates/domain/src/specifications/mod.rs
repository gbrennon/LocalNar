//! Rules stated as a yes-or-no question about a single candidate.
//!
//! Each rule judges one candidate in isolation and composes with the
//! others by boolean logic, so a rule can be reused and tested without
//! the code that applies it. A rule that must compare candidates belongs
//! in `policies`.

mod multi_part_shard;
mod specification;
mod whole_weight_file;

pub use multi_part_shard::MultiPartShard;
pub use specification::Specification;
pub use whole_weight_file::WholeWeightFile;
