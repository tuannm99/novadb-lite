# novadb-lite

Rust workspace for a page-based database prototype.

## Workspace Layout

This repo is split into Rust crates under `crates/`:

- `crates/storage`: storage engine primitives and current working implementation.
- `crates/planner`: planner crate boundary.
- `crates/executor`: executor crate boundary.
- `crates/cache`: cache crate boundary.

Current implementation work is concentrated in `crates/storage`.

## Storage Scope

The storage crate currently covers:

- Single-file page storage.
- Slotted-page layout.
- Fixed-size page I/O through a pager abstraction.
- Unit tests inside each Rust module.
- Integration tests for cross-module behavior through the public API.

## Run Commands

All commands below are run from the workspace root:

```bash
cargo test
```

Runs every crate in the workspace, including integration tests.

```bash
cargo test -p novadb-storage
```

Runs only the `storage` crate tests.

```bash
cargo test -p novadb-storage --lib
```

Runs only unit tests inside `crates/storage/src/*.rs`.

```bash
cargo test -p novadb-storage --test storage_integration
```

Runs only the storage integration test in `crates/storage/tests/storage_integration.rs`.

```bash
cargo test -p novadb-planner
cargo test -p novadb-executor
cargo test -p novadb-cache
```

Runs tests for one workspace crate at a time.

## Docs

Generate docs for the whole workspace:

```bash
cargo doc --workspace --no-deps
```

Generate docs for one crate only:

```bash
cargo doc -p novadb-storage --no-deps
```

Open generated docs locally in a browser:

```bash
cargo doc -p novadb-storage --no-deps --open
```

If you do not want `cargo` to open the browser, generated files are under:

```text
target/doc/
```

The main storage crate entry is typically:

```text
target/doc/novadb_storage/index.html
```

## Test Layout

Unit tests live next to implementation code:

- `crates/storage/src/raw.rs`
- `crates/storage/src/page_header.rs`
- `crates/storage/src/page_slot.rs`
- `crates/storage/src/slotted_page.rs`
- `crates/storage/src/file_pager.rs`

Integration tests live outside the modules in the standard Rust location:

- `crates/storage/tests/storage_integration.rs`

## Public API Notes

The crate is structured so that:

- Public API is re-exported from `crates/storage/src/lib.rs`.
- Internal helpers stay internal unless they are required across storage modules.
- Doc comments are added on the exposed types, traits, constants, and methods.
