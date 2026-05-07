/// Fixed database page size in bytes.
pub const PAGE_SIZE: usize = 4096;
/// On-disk database magic prefix.
pub const DB_MAGIC: &[u8; 12] = b"NOVADBLITE\0\0";
/// Current storage format version.
pub const DB_VERSION: u16 = 1;

/// Size of the slotted page header in bytes.
pub const SLOTTED_HEADER_SIZE: usize = 16;
/// Size of a single slot entry in bytes.
pub const SLOTTED_SLOT_SIZE: usize = 6;

/// Heap page type tag.
pub const PAGE_TYPE_HEAP: u16 = 0;
/// B-tree leaf page type tag.
pub const PAGE_TYPE_BTREE_LEAF: u16 = 1;
/// B-tree internal page type tag.
pub const PAGE_TYPE_BTREE_INTERNAL: u16 = 2;
/// Overflow page type tag.
pub const PAGE_TYPE_BTREE_OVERFLOW: u16 = 3;

const FLAG_HAS_FREE_SLOTS_BIT: u16 = 4;
const FLAG_IS_COMPRESSED_BIT: u16 = 5;
const FLAG_IS_CHECKSUMMED_BIT: u16 = 6;

/// Header flag indicating that at least one slot tombstone can be reused.
pub const FLAG_HAS_FREE_SLOTS: u16 = 1 << FLAG_HAS_FREE_SLOTS_BIT;
/// Header flag reserved for future page compression support.
pub const FLAG_IS_COMPRESSED: u16 = 1 << FLAG_IS_COMPRESSED_BIT;
/// Header flag reserved for future checksum support.
pub const FLAG_IS_CHECKSUMMED: u16 = 1 << FLAG_IS_CHECKSUMMED_BIT;
