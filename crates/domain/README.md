# localnar-domain

The framework-free core of LocalNar. This crate models what a *local model
library* **is** and the rules that govern it. It is the innermost layer of the
application: everything else (the application/use-case layer, the infrastructure
adapters, and the TUI) depends on this crate, and this crate depends on none of
them.

## What this crate is responsible for

- **Naming the concepts.** The vocabulary the rest of the codebase speaks —
  what a "model", a "replica", an "install intent", or a "capability tag" is.
- **Enforcing invariants at construction.** A value object validates itself when
  it is built, so an invalid value (a blank tag, a malformed repository id, an
  empty search phrase) cannot exist. Callers handle a [`DomainError`] once, at
  the boundary, instead of re-checking downstream.
- **Stating the rules.** Pure decisions (policies) and pure predicates
  (specifications) that judge domain values without any I/O.
- **Describing the model lifecycle.** The states a model moves through, from a
  free-text search phrase to a verified replica on disk.

## What this crate is deliberately *not* responsible for

- **No I/O.** No network, no filesystem, no database, no environment variables.
  A checksum is *described* here; the bytes are hashed elsewhere.
- **No frameworks or runtimes.** No `reqwest`, no `tokio`, no `ratatui`, no
  serialization derives. Adapters translate the outside world into these types.
- **No use-case orchestration.** Sequencing "search, then download, then verify"
  is the application layer's job. This crate only supplies the pieces it moves.
- **No catalog-specific taxonomy.** Concepts like Hugging Face's `likes` or
  download counts stay in the infrastructure adapters. Even open concepts such
  as capability tags are kept vocabulary-free here (see [`ModelTag`]).

If a type here starts needing a URL, a file handle, or a JSON derive, it belongs
in an adapter, not in this crate.

## The models, and what each one represents

"Model" is overloaded in this domain, so each type answers one precise question.
Read them as stages of a single lifecycle, from a typed phrase to a verified
replica on disk:

- **Discover** — [`SearchQuery`]: what is the operator looking for?
- **Discover** — [`RemoteModelFile`]: what does the catalog offer, as the domain
  sees it?
- **Discover** — [`ModelInfo`]: how is one candidate shown to the operator?
- **Intend** — [`ModelSpec`]: which exact model does the operator mean to
  install?
- **Stage** — [`ModelArtifact`]: where are the freshly downloaded,
  not-yet-committed bytes?
- **Install** — [`InstalledModel`]: where did the file land in the durable
  library?
- **Manage** — [`ManagedModel`]: what is that replica, together with how much it
  can be trusted?
- **Manage** — [`ModelInventory`]: everything the library holds, read as one
  answer.

### Which type do the services actually use?

- **[`ModelSpec`] is the identity of a model.** A repository paired with a file
  names exactly one downloadable model. It is the self-contained *install
  intent*: a search result turns into a `ModelSpec` with nothing further to ask
  the operator. Capability [`ModelTag`]s travel alongside the identity but take
  no part in equality — a model is the same model regardless of how it was
  labeled.
- **[`ManagedModel`] is "the model as the library holds it."** It pairs the
  durable [`InstalledModel`] (where the bytes are) with a [`ModelState`] (how
  much they can be trusted). This is what the *manage/inspect/verify/remove*
  use cases operate on.
- **[`ModelInfo`] is "the model as a search hit."** It is a read-only
  presentation of one candidate, built from the chosen weight file alone, so its
  size, quantization, and tags can never disagree with the [`ModelSpec`] it
  carries.

In short: **`ModelSpec` = identity/intent, `ManagedModel` = the durable model in
context, `ModelInfo` = the discoverable candidate.** The others are supporting
values on the path between them.

## Building blocks

The crate is organized by the *kind* of concept, not by feature:

### `value_objects/`

Values identified by what they hold rather than by identity. Each is immutable,
compares by contents, and validates itself on construction.

- Identity & location: [`ModelRepositoryId`], [`ModelRevision`],
  [`ModelRepository`], [`ModelFileName`], [`ModelSpec`].
- Discovery: [`SearchQuery`], [`RemoteModelFile`], [`ModelInfo`],
  [`ModelProfile`].
- Facts & capabilities: [`ByteLength`], [`Checksum`], [`ContextLength`],
  [`ParameterCount`], [`Quantization`], [`ModelTag`].
- Lifecycle state: [`ModelState`], [`ModelArtifact`].

### `entities/`

Things identified by *who they are* rather than by what they hold; an entity
keeps its identity across changes to its other attributes.

- [`InstalledModel`], [`ManagedModel`], [`ModelInventory`], [`RemovedModel`],
  [`DiscardedStray`].

### `policies/`

Rules that decide something without holding state of their own; the same inputs
always yield the same decision.

- [`ModelWeightChoice`] — reduces everything a repository offers to the one
  candidate file that stands for the model.

### `specifications/`

Rules stated as a yes-or-no question about a single candidate. Each judges one
candidate in isolation and composes with the others by boolean logic.

- [`Specification`] (the contract), [`WholeWeightFile`], [`MultiPartShard`].

### `errors/`

- [`DomainError`] — every way a domain rule can reject a value or invariant.

## Capability tags

A model is *marked with* capability [`ModelTag`]s (for example
`text-generation`, `conversational`). Tags are descriptive: they travel with a
model's identity but are excluded from it. [`ModelSpec`] equality and hashing
consider only the repository and file, so two intents for the same model compare
equal and hash alike however they were tagged — a spec-keyed lookup never splits
a model apart by the adjectives a catalog happened to attach.

Tags flow along the lifecycle without being re-derived:

```
RemoteModelFile::with_tags(..)  ->  RemoteModelFile::to_spec()  ->  ModelSpec
        |                                                              |
        +--> ModelInfo::describing(..)  --(ModelInfo::tags)-->        |
                                                                       v
                          InstalledModel / ManagedModel (spec carries the tags)
```

Because the tags live on the [`ModelSpec`] the file produces, a model discovered
in search keeps exactly the capabilities it will still report once installed.
The vocabulary is intentionally open — any non-blank label is a valid tag — so
the domain stays free of any single catalog's taxonomy.

## Conventions

- **Docstrings, not comments.** Public items are documented with `///` (and
  modules with `//!`). Implementation is written to be read without inline `//`
  commentary.
- **Tests live beside the code** in a `#[cfg(test)]` module named after the unit
  under test, phrased as statements about behavior.
- **Construction validates.** Prefer returning `Result<_, DomainError>` from
  constructors over letting an invalid value exist.

[`ByteLength`]: src/value_objects/byte_length.rs
[`Checksum`]: src/value_objects/checksum.rs
[`ContextLength`]: src/value_objects/context_length.rs
[`DiscardedStray`]: src/entities/discarded_stray.rs
[`DomainError`]: src/errors/domain_error.rs
[`InstalledModel`]: src/entities/installed_model.rs
[`ManagedModel`]: src/entities/managed_model.rs
[`ModelArtifact`]: src/value_objects/model_artifact.rs
[`ModelFileName`]: src/value_objects/model_file_name.rs
[`ModelInfo`]: src/value_objects/model_info.rs
[`ModelInventory`]: src/entities/model_inventory.rs
[`ModelProfile`]: src/value_objects/model_profile.rs
[`ModelRepository`]: src/value_objects/model_repository.rs
[`ModelRepositoryId`]: src/value_objects/model_repository_id.rs
[`ModelRevision`]: src/value_objects/model_revision.rs
[`ModelSpec`]: src/value_objects/model_spec.rs
[`ModelState`]: src/value_objects/model_state.rs
[`ModelTag`]: src/value_objects/model_tag.rs
[`ModelWeightChoice`]: src/policies/model_weight_choice.rs
[`MultiPartShard`]: src/specifications/multi_part_shard.rs
[`ParameterCount`]: src/value_objects/parameter_count.rs
[`Quantization`]: src/value_objects/quantization.rs
[`RemoteModelFile`]: src/value_objects/remote_model_file.rs
[`RemovedModel`]: src/entities/removed_model.rs
[`SearchQuery`]: src/value_objects/search_query.rs
[`Specification`]: src/specifications/specification.rs
[`WholeWeightFile`]: src/specifications/whole_weight_file.rs
