/// Page identifier inside a single database file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageId(pub u32);

impl PageId {
    /// Reserved invalid page id.
    pub const INVALID: PageId = PageId(u32::MAX);

    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0 as u64
    }

    #[inline]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
