use super::{SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};

/// bitmask value
/// 0: DELETED
/// 1: REDIRECTED
/// 2: OVERFLOW
/// 3..8 -> reserved - mở rộng nếu có thể
const SLOT_DEAD: u16 = 1 << 0;
const SLOT_REDIRECTED: u16 = 1 << 1;
const SLOT_OVERFLOW: u16 = 1 << 2;

/// slot size = 6
/// slot(i) = HEADER_SIZE + i*SLOT_SIZE
pub struct Slot {
    offset: u16,
    len: u16,
    flags: u16,
}

impl Slot {
    pub fn offset(&self) -> u16 {
        return self.offset;
    }

    pub fn len(&self) -> u16 {
        return self.len;
    }

    pub fn flags(&self) -> u16 {
        return self.flags;
    }
}

fn slot_off(i: usize) -> usize {
    return SLOTTED_HEADER_SIZE + i * SLOTTED_SLOT_SIZE;
}
