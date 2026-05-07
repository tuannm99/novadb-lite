use crate::error::Result;
use crate::types::PageId;

/// Abstract interface for fixed-size page persistence.
pub trait Pager {
    /// Reads a page into `out`.
    fn read_page(&mut self, pid: PageId, out: &mut [u8]) -> Result<()>;
    /// Writes a full page from `buf`.
    fn write_page(&mut self, pid: PageId, buf: &[u8]) -> Result<()>;
    /// Allocates and zero-initializes a new page, or reuses a freed page id.
    fn alloc_page(&mut self) -> Result<PageId>;
    /// Returns a page id to the pager freelist.
    fn free_page(&mut self, pid: PageId) -> Result<()>;
    /// Flushes durable state to the underlying storage medium.
    fn flush(&mut self) -> Result<()>;
    /// Returns the number of pages currently visible in the backing store.
    fn num_pages(&self) -> Result<u64>;
}
