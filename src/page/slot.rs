use crate::{DbError, DbResult};

use super::raw::{read_u16_le, write_u16_le};
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

fn current_pos(buf: &[u8], slot_id: u16) -> DbResult<usize> {
    let base = slot_off(slot_id);
    if base + SLOTTED_SLOT_SIZE > buf.len() {
        return Err(DbError::Corruption("slot entry out of bounds"));
    }
    Ok(base)
}

pub fn read_slot(buf: &[u8], slot_id: u16) -> DbResult<Slot> {
    let pos = current_pos(buf, slot_id)?;

    let offset = read_u16_le(buf, pos + OFF_SLOT_OFFSET)?;
    let len = read_u16_le(buf, pos + OFF_SLOT_LEN)?;
    let flags = read_u16_le(buf, pos + OFF_SLOT_FLAGS)?;

    Ok(Slot { offset, len, flags })
}

pub fn write_slot(buf: &mut [u8], slot_id: u16, slot: &Slot) -> DbResult<()> {
    let pos = current_pos(buf, slot_id)?;

    write_u16_le(buf, pos + OFF_SLOT_OFFSET, slot.offset)?;
    write_u16_le(buf, pos + OFF_SLOT_LEN, slot.len)?;
    write_u16_le(buf, pos + OFF_SLOT_FLAGS, slot.flags)?;

    Ok(())
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
mod tests {
    use super::*;
    use crate::constants::PAGE_SIZE;

    #[test]
    fn test_read_write_roundtrip() {
        let mut buf = vec![0u8; PAGE_SIZE];

        let slot = Slot {
            offset: 123,
            len: 45,
            flags: 0x0002,
        };

        write_slot(&mut buf, 0, &slot).unwrap();
        let got = read_slot(&buf, 0).unwrap();

        assert_eq!(got, slot);
    }

    #[test]
    fn test_slot_out_of_bounds() {
        let mut buf = vec![0u8; PAGE_SIZE];
        let slot = Slot {
            offset: 1,
            len: 1,
            flags: 0,
        };

        // slot_id cực lớn => base vượt page
        assert!(write_slot(&mut buf, u16::MAX, &slot).is_err());
        assert!(read_slot(&buf, u16::MAX).is_err());
    }

    #[test]
    fn test_is_dead() {
        assert!(is_dead(1 << 0));
        assert!(!is_dead(0));
    }

    #[test]
    fn test_flags_helpers() {
        assert!(is_redirected(1 << 1));
        assert!(is_overflow(1 << 2));
        assert!(!is_redirected(0));
        assert!(!is_overflow(0));
    }
}
