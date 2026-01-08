use crate::DbResult;

use super::{SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};

/// bitmask value
/// 0: DELETED
/// 1: REDIRECTED
/// 2: OVERFLOW
/// 3..8 -> reserved - mở rộng nếu có thể
const SLOT_DEAD: u16 = 1 << 0;
const SLOT_REDIRECTED: u16 = 1 << 1;
const SLOT_OVERFLOW: u16 = 1 << 2;

// fixed position cho mỗi slot
const OFF_SLOT_OFFSET: usize = 0;
const OFF_SLOT_LEN: usize = 2;
const OFF_SLOT_FLAGS: usize = 4;

/// slot size = 6
/// slot(i) = HEADER_SIZE + i*SLOT_SIZE
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    offset: u16,
    len: u16,
    flags: u16,
}

impl Slot {
    pub fn offset(&self) -> u16 {
        self.offset
    }

    pub fn len(&self) -> u16 {
        self.len
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }
}

pub fn slot_off(slot_id: u16) -> usize {
    SLOTTED_HEADER_SIZE + slot_id as usize * SLOTTED_SLOT_SIZE
}

pub fn read_slot(buf: &[u8], slot_id: u16) -> DbResult<Slot> {
    // NEED CHECK: base + SLOTTED_SLOT_SIZE <= buf.len()
    todo!()
}

pub fn write_slot(buf: &mut [u8], slot_id: u16, slot: &Slot) -> DbResult<()> {
    // NEED CHECK: base + SLOTTED_SLOT_SIZE <= buf.len()
    todo!()
}

pub fn is_dead(flags: u16) -> bool {
    flags & SLOT_DEAD != 0
}

pub fn is_redirected(flags: u16) -> bool {
    flags & SLOT_REDIRECTED != 0
}

pub fn is_overflow(flags: u16) -> bool {
    flags & SLOT_OVERFLOW != 0
}

#[cfg(test)]
mod tests {}
