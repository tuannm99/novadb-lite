/// Stable identifier for a fixed-size page in the storage layer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PageId(pub u32);

impl PageId {
    /// Sentinel value representing an invalid page id.
    pub const INVALID: Self = Self(u32::MAX);

    /// Returns the raw `u32` representation.
    pub fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns the raw value widened to `u64`.
    pub fn as_u64(self) -> u64 {
        self.0 as u64
    }

    /// Returns the raw value widened to `usize`.
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
