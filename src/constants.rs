// Global constants for the storage engine.
// Keep this file small and stable.

pub const PAGE_SIZE: usize = 4096;

// 12 bytes magic header
pub const DB_MAGIC: [u8; 12] = *b"NOVADBLITE\0\0";

pub const DB_VERSION: u16 = 1;
