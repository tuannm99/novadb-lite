#![cfg_attr(not(test), allow(dead_code))]

//! Storage primitives for page-based persistence and slotted-page layout.

mod error;
mod file_pager;
mod page_header;
mod page_slot;
mod pager;
mod raw;
mod slotted_page;
mod types;

pub mod constants;

/// Re-exported storage constants and page layout flags.
pub use constants::{
    DB_MAGIC, DB_VERSION, FLAG_HAS_FREE_SLOTS, FLAG_IS_CHECKSUMMED, FLAG_IS_COMPRESSED,
    PAGE_SIZE, PAGE_TYPE_BTREE_INTERNAL, PAGE_TYPE_BTREE_LEAF, PAGE_TYPE_BTREE_OVERFLOW,
    PAGE_TYPE_HEAP, SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE,
};
/// Error type and result alias used across the storage crate.
pub use error::{DbError, ErrorKind, Result};
/// File-backed pager implementation.
pub use file_pager::FilePager;
/// Decoded view of a slotted page header.
pub use page_header::PageHeaderSnapshot;
/// Pager abstraction for reading and writing fixed-size pages.
pub use pager::Pager;
/// Mutable view over a slotted page buffer.
pub use slotted_page::SlottedPage;
/// Logical page identifier.
pub use types::PageId;
