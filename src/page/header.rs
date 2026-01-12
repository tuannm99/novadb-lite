use crate::constants::PAGE_SIZE;
use crate::page::raw::{read_u16_le, read_u64_le, write_u16_le, write_u64_le};
use crate::page::SLOTTED_HEADER_SIZE;
use crate::{DbError, DbResult};

const OFF_LOWER: usize = 0;
const OFF_UPPER: usize = 2;
const OFF_SLOT_COUNT: usize = 4;
const OFF_FLAGS: usize = 6;
const OFF_RESERVED: usize = 8;

pub const PAGE_TYPE_HEAP: u16 = 0;
pub const PAGE_TYPE_BTREE_LEAF: u16 = 1;
pub const PAGE_TYPE_BTREE_INTERNAL: u16 = 2;
pub const PAGE_TYPE_BTREE_OVERFLOW: u16 = 3;

pub const FLAG_HAS_FREE_SLOTS_BIT: u16 = 4;
pub const FLAG_IS_COMPRESSED_BIT: u16 = 5;
pub const FLAG_IS_CHECKSUMMED_BIT: u16 = 6;

pub const FLAG_HAS_FREE_SLOTS: u16 = 1u16 << FLAG_HAS_FREE_SLOTS_BIT;
pub const FLAG_IS_COMPRESSED: u16 = 1u16 << FLAG_IS_COMPRESSED_BIT;
pub const FLAG_IS_CHECKSUMMED: u16 = 1u16 << FLAG_IS_CHECKSUMMED_BIT;

/// page header fixed 16 bytes, trong đó có 8 bytes là reserved
/// PageHeader chỉ biểu diễn dữ liệu được lưu trong program, chứ k phải layout dưới disk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageHeaderSnapshot {
    /// lower >= HEADER_SIZE (16)
    /// upper <= PAGE_SIZE
    /// lower <= upper
    lower: u16,
    upper: u16,

    /// slot_count * SLOT_SIZE + HEADER_SIZE == lower
    /// slot_count là số slot đã cấp phát (không giảm), slot_id < slot_count
    slot_count: u16,

    /// flags: bitmask trạng thái ở cấp PAGE
    ///
    /// - Bits 0..3  : page_type (0=heap, 1=btree_leaf, 2=btree_internal, 3=overflow, 4..15 reserved)
    /// - Bit  4     : HAS_FREE_SLOTS (trang có slot tombstone để reuse)
    /// - Bit  5     : IS_COMPRESSED (nếu sau này có nén)
    /// - Bit  6     : IS_CHECKSUMMED (nếu bật checksum)
    /// - Bit  7     : RESERVED
    /// - Bits 8..15 : mở rộng sau
    flags: u16,

    /// special field, mở rộng sau này (lsn, checksum, future metadata...)
    reserved: u64,
}

impl PageHeaderSnapshot {
    pub fn upper(&self) -> u16 {
        self.upper
    }

    pub fn lower(&self) -> u16 {
        self.lower
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }

    pub fn slot_count(&self) -> u16 {
        self.slot_count
    }

    pub fn reserved(&self) -> u64 {
        self.reserved
    }
}

pub fn decode(buf: &[u8]) -> DbResult<PageHeaderSnapshot> {
    if buf.len() != PAGE_SIZE {
        return Err(DbError::Corruption("buffer length must equal PAGE_SIZE"));
    }

    Ok(PageHeaderSnapshot {
        lower: read_u16_le(buf, OFF_LOWER)?,
        upper: read_u16_le(buf, OFF_UPPER)?,
        slot_count: read_u16_le(buf, OFF_SLOT_COUNT)?,
        flags: read_u16_le(buf, OFF_FLAGS)?,
        reserved: read_u64_le(buf, OFF_RESERVED)?,
    })
}

/// các public function thể hiện view đọc/ghi header trực tiếp trên page bytes (on-disk layout)
/// đổi sang KHÔNG sử dụng struct vì chưa muốn mess với lifetime trong rust
pub fn init_empty(buf: &mut [u8], page_type: u16) -> DbResult<()> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);

    let flags = page_type & 0x000F;
    set_lower(buf, SLOTTED_HEADER_SIZE as u16)?;
    set_upper(buf, PAGE_SIZE as u16)?;
    set_slot_count(buf, 0)?;
    set_flags(buf, flags)?;
    set_reserved(buf, 0)?;
    Ok(())
}

pub fn lower(buf: &[u8]) -> DbResult<u16> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    read_u16_le(buf, OFF_LOWER)
}
pub fn set_lower(buf: &mut [u8], v: u16) -> DbResult<()> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    write_u16_le(buf, OFF_LOWER, v)
}
pub fn upper(buf: &[u8]) -> DbResult<u16> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    read_u16_le(buf, OFF_UPPER)
}
pub fn set_upper(buf: &mut [u8], v: u16) -> DbResult<()> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    write_u16_le(buf, OFF_UPPER, v)
}

pub fn slot_count(buf: &[u8]) -> DbResult<u16> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    read_u16_le(buf, OFF_SLOT_COUNT)
}

pub fn set_slot_count(buf: &mut [u8], v: u16) -> DbResult<()> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    write_u16_le(buf, OFF_SLOT_COUNT, v)
}
pub fn flags(buf: &[u8]) -> DbResult<u16> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    read_u16_le(buf, OFF_FLAGS)
}
pub fn set_flags(buf: &mut [u8], v: u16) -> DbResult<()> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    write_u16_le(buf, OFF_FLAGS, v)
}
pub fn reserved(buf: &[u8]) -> DbResult<u64> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    read_u64_le(buf, OFF_RESERVED)
}
pub fn set_reserved(buf: &mut [u8], v: u64) -> DbResult<()> {
    debug_assert_eq!(buf.len(), PAGE_SIZE);
    write_u64_le(buf, OFF_RESERVED, v)
}

pub fn is_page_type(flags: u16, t: u16) -> bool {
    (flags & 0x000F) == (t & 0x000F)
}

pub fn set_page_type(flags: u16, t: u16) -> u16 {
    (flags & !0x000F) | (t & 0x000F)
}

pub fn has_free_slots(flags: u16) -> bool {
    (flags & FLAG_HAS_FREE_SLOTS) != 0
}

pub fn set_flag(flags: u16, mask: u16) -> u16 {
    flags | mask
}
pub fn clear_flag(flags: u16, mask: u16) -> u16 {
    flags & !mask
}
pub fn has_flag(flags: u16, mask: u16) -> bool {
    (flags & mask) != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};

    fn new_page_buf() -> Vec<u8> {
        vec![0u8; PAGE_SIZE]
    }

    fn check_invariants(h: &PageHeaderSnapshot) {
        let header_size = SLOTTED_HEADER_SIZE as u16;
        let slot_size = SLOTTED_SLOT_SIZE as u16;
        let page_size = PAGE_SIZE as u16;

        assert!(h.lower() >= header_size);
        assert!(h.upper() <= page_size);
        assert!(h.lower() <= h.upper());

        let expected_lower = header_size + h.slot_count() * slot_size;
        assert_eq!(h.lower(), expected_lower);
    }

    #[test]
    fn test_flags_helpers_and_set_page_type() {
        let flags = set_flag(PAGE_TYPE_BTREE_INTERNAL, FLAG_HAS_FREE_SLOTS);

        assert!(is_page_type(flags, PAGE_TYPE_BTREE_INTERNAL));
        assert!(has_free_slots(flags));

        let flags2 = set_page_type(flags, PAGE_TYPE_HEAP);
        assert!(is_page_type(flags2, PAGE_TYPE_HEAP));
        assert!(has_free_slots(flags2)); // must preserve non-type bits
    }

    #[test]
    fn test_flag_helpers_generic() {
        let mut f: u16 = 0;
        assert!(!has_flag(f, FLAG_HAS_FREE_SLOTS));

        f = set_flag(f, FLAG_HAS_FREE_SLOTS);
        assert!(has_flag(f, FLAG_HAS_FREE_SLOTS));

        f = clear_flag(f, FLAG_HAS_FREE_SLOTS);
        assert!(!has_flag(f, FLAG_HAS_FREE_SLOTS));
    }

    #[test]
    fn test_invariants_ok() {
        let slot_count: u16 = 10;
        let lower = SLOTTED_HEADER_SIZE as u16 + slot_count * SLOTTED_SLOT_SIZE as u16;

        let h = PageHeaderSnapshot {
            lower,
            upper: PAGE_SIZE as u16,
            slot_count,
            flags: 0,
            reserved: 0,
        };

        check_invariants(&h);
    }

    #[test]
    #[should_panic]
    fn test_invariants_fail_lower_formula() {
        let h = PageHeaderSnapshot {
            lower: (SLOTTED_HEADER_SIZE as u16) + 1, // wrong formula
            upper: PAGE_SIZE as u16,
            slot_count: 0,
            flags: 0,
            reserved: 0,
        };

        check_invariants(&h);
    }

    #[test]
    fn test_init_empty_sets_fields() {
        let mut buf = new_page_buf();
        init_empty(&mut buf, PAGE_TYPE_BTREE_INTERNAL).unwrap();

        assert_eq!(lower(&buf).unwrap(), SLOTTED_HEADER_SIZE as u16);
        assert_eq!(upper(&buf).unwrap(), PAGE_SIZE as u16);
        assert_eq!(slot_count(&buf).unwrap(), 0);
        assert!(is_page_type(flags(&buf).unwrap(), PAGE_TYPE_BTREE_INTERNAL));
        assert_eq!(reserved(&buf).unwrap(), 0);
    }

    #[test]
    fn test_header_setters_roundtrip() {
        let mut buf = new_page_buf();
        init_empty(&mut buf, PAGE_TYPE_HEAP).unwrap();

        set_lower(&mut buf, 123).unwrap();
        set_upper(&mut buf, 4000).unwrap();
        set_slot_count(&mut buf, 10).unwrap();
        set_flags(&mut buf, 0x00F2).unwrap();
        set_reserved(&mut buf, 0x1122_3344_5566_7788).unwrap();

        assert_eq!(lower(&buf).unwrap(), 123);
        assert_eq!(upper(&buf).unwrap(), 4000);
        assert_eq!(slot_count(&buf).unwrap(), 10);
        assert_eq!(flags(&buf).unwrap(), 0x00F2);
        assert_eq!(reserved(&buf).unwrap(), 0x1122_3344_5566_7788);
    }

    #[test]
    fn test_decode_invalid_size() {
        let buf = vec![0u8; 100];
        let e = decode(&buf).is_err();
        assert_eq!(e, true)
    }

    #[test]
    fn test_decode_roundtrip_basic() {
        let mut buf = new_page_buf();
        init_empty(&mut buf, PAGE_TYPE_BTREE_LEAF).unwrap();
        let cur = flags(&buf).unwrap();
        set_flags(&mut buf, set_flag(cur, FLAG_HAS_FREE_SLOTS)).unwrap();
        set_reserved(&mut buf, 99).unwrap();

        let h = decode(&buf).unwrap();
        assert_eq!(h.lower(), SLOTTED_HEADER_SIZE as u16);
        assert_eq!(h.upper(), PAGE_SIZE as u16);
        assert_eq!(h.slot_count(), 0);
        assert!(is_page_type(h.flags(), PAGE_TYPE_BTREE_LEAF));
        assert!(has_free_slots(h.flags()));
        assert_eq!(h.reserved(), 99);
    }

    #[test]
    fn test_struct_size_sanity() {
        assert_eq!(std::mem::size_of::<PageHeaderSnapshot>(), 16);
    }
}
