use crate::constants::{SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};
use crate::error::{DbError, Result};
use crate::raw::{read_u16_le, write_u16_le};

const SLOT_DEAD: u16 = 1 << 0;
const SLOT_REDIRECTED: u16 = 1 << 1;
const SLOT_OVERFLOW: u16 = 1 << 2;

const OFF_SLOT_OFFSET: usize = 0;
const OFF_SLOT_LEN: usize = 2;
const OFF_SLOT_FLAGS: usize = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Slot {
    offset: u16,
    length: u16,
    flags: u16,
}

impl Slot {
    fn new(offset: u16, length: u16, flags: u16) -> Self {
        Self {
            offset,
            length,
            flags,
        }
    }

    fn offset(self) -> u16 {
        self.offset
    }

    fn len(self) -> u16 {
        self.length
    }

    fn flags(self) -> u16 {
        self.flags
    }

    fn mark_dead(&mut self) {
        self.flags |= SLOT_DEAD;
    }

    fn mark_redirected(&mut self) {
        self.flags |= SLOT_REDIRECTED;
    }

    fn mark_overflow(&mut self) {
        self.flags |= SLOT_OVERFLOW;
    }
}

fn slot_off(slot_id: u16) -> usize {
    SLOTTED_HEADER_SIZE + (slot_id as usize) * SLOTTED_SLOT_SIZE
}

fn current_pos(buf: &[u8], slot_id: u16) -> Result<usize> {
    let base = slot_off(slot_id);
    if base + SLOTTED_SLOT_SIZE > buf.len() {
        return Err(DbError::corruption("slot entry out of bounds"));
    }
    Ok(base)
}

pub(crate) fn read_slot(buf: &[u8], slot_id: u16) -> Result<Slot> {
    let pos = current_pos(buf, slot_id)?;
    Ok(Slot {
        offset: read_u16_le(buf, pos + OFF_SLOT_OFFSET)?,
        length: read_u16_le(buf, pos + OFF_SLOT_LEN)?,
        flags: read_u16_le(buf, pos + OFF_SLOT_FLAGS)?,
    })
}

pub(crate) fn write_slot(buf: &mut [u8], slot_id: u16, slot: Slot) -> Result<()> {
    let pos = current_pos(buf, slot_id)?;
    write_u16_le(buf, pos + OFF_SLOT_OFFSET, slot.offset)?;
    write_u16_le(buf, pos + OFF_SLOT_LEN, slot.length)?;
    write_u16_le(buf, pos + OFF_SLOT_FLAGS, slot.flags)?;
    Ok(())
}

pub(crate) fn is_dead(flags: u16) -> bool {
    flags & SLOT_DEAD != 0
}

pub(crate) fn dead_slot(mut slot: Slot) -> Slot {
    slot.mark_dead();
    slot
}

pub(crate) fn slot(offset: u16, length: u16, flags: u16) -> Slot {
    Slot::new(offset, length, flags)
}

pub(crate) fn slot_fields(slot: Slot) -> (u16, u16, u16) {
    (slot.offset(), slot.len(), slot.flags())
}

fn is_redirected(flags: u16) -> bool {
    flags & SLOT_REDIRECTED != 0
}

fn is_overflow(flags: u16) -> bool {
    flags & SLOT_OVERFLOW != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::PAGE_SIZE;

    #[test]
    fn slot_read_write_roundtrip() {
        let mut buf = vec![0; PAGE_SIZE];
        let slot = Slot {
            offset: 123,
            length: 45,
            flags: 0x0002,
        };
        write_slot(&mut buf, 0, slot).unwrap();
        assert_eq!(slot, read_slot(&buf, 0).unwrap());
    }

    #[test]
    fn slot_out_of_bounds() {
        let mut buf = vec![0; PAGE_SIZE];
        let slot = Slot {
            offset: 1,
            length: 1,
            flags: 0,
        };
        assert!(write_slot(&mut buf, u16::MAX, slot).is_err());
        assert!(read_slot(&buf, u16::MAX).is_err());
    }

    #[test]
    fn slot_flag_helpers() {
        assert!(is_dead(1 << 0));
        assert!(!is_dead(0));
        assert!(is_redirected(1 << 1));
        assert!(is_overflow(1 << 2));
        assert!(!is_redirected(0));
        assert!(!is_overflow(0));
    }

    #[test]
    fn slot_markers() {
        let mut slot = Slot::default();
        slot.mark_redirected();
        slot.mark_overflow();
        assert!(is_redirected(slot.flags()));
        assert!(is_overflow(slot.flags()));
    }
}
