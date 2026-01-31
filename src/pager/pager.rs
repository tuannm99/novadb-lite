use crate::{DbError, DbResult, PageId};

pub trait Pager {
    fn read_page(pid: PageId, out: &mut [u8]) -> DbResult<()>;
    fn write_page(pid: PageId, buf: &[u8]) -> Result<(), DbError>;
    fn alloc_page() -> DbResult<PageId>;
    fn free_page(&mut self, pid: PageId) -> DbResult<()>;
    fn flush(&mut self) -> DbResult<()>;
    fn num_pages(&self);
}
