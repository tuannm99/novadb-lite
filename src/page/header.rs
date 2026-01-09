use crate::constants::PAGE_SIZE;
use crate::page::raw::{read_u16_le, read_u64_le, write_u16_le, write_u64_le};
use crate::page::SLOTTED_HEADER_SIZE;
use crate::{DbError, DbResult};

const OFF_LOWER: usize = 0;
const OFF_UPPER: usize = 2;
const OFF_SLOT_COUNT: usize = 4;
const OFF_FLAGS: usize = 6;
const OFF_RESERVED: usize = 8;

/// page header fixed 16 bytes, trong đó có 8 bytes là reserved
/// PageHeader chỉ biểu diễn dữ liệu được lưu trong program, chứ k phải layout dưới disk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageHeader {
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

impl PageHeader {
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

/// các public function thể hiện view đọc/ghi header trực tiếp trên page bytes (on-disk layout)
/// đổi sang KHÔNG sử dụng struct vì chưa muốn mess với lifetime trong rust
pub fn init_empty(buf: &mut [u8], page_type: u16) -> DbResult<()> {
    validate(buf)?;

    let flags = page_type & 0x000F;
    set_lower(buf, SLOTTED_HEADER_SIZE as u16)?;
    set_upper(buf, PAGE_SIZE as u16)?;
    set_slot_count(buf, 0)?;
    set_flags(buf, flags)?;
    set_reserved(buf, 0)?;
    Ok(())
}

pub fn validate(buf: &[u8]) -> DbResult<()> {
    if buf.len() != PAGE_SIZE {
        return Err(DbError::Corruption("buffer length must equal PAGE_SIZE"));
    }
    Ok(())
}

pub fn lower(buf: &[u8]) -> DbResult<u16> {
    validate(buf)?;
    read_u16_le(buf, OFF_LOWER)
}
pub fn set_lower(buf: &mut [u8], v: u16) -> DbResult<()> {
    validate(buf)?;
    write_u16_le(buf, OFF_LOWER, v)
}
pub fn upper(buf: &[u8]) -> DbResult<u16> {
    validate(buf)?;
    read_u16_le(buf, OFF_UPPER)
}
pub fn set_upper(buf: &mut [u8], v: u16) -> DbResult<()> {
    validate(buf)?;
    write_u16_le(buf, OFF_UPPER, v)
}

pub fn slot_count(buf: &[u8]) -> DbResult<u16> {
    validate(buf)?;
    read_u16_le(buf, OFF_SLOT_COUNT)
}

pub fn set_slot_count(buf: &mut [u8], v: u16) -> DbResult<()> {
    validate(buf)?;
    write_u16_le(buf, OFF_SLOT_COUNT, v)
}
pub fn flags(buf: &[u8]) -> DbResult<u16> {
    validate(buf)?;
    read_u16_le(buf, OFF_FLAGS)
}
pub fn set_flags(buf: &mut [u8], v: u16) -> DbResult<()> {
    validate(buf)?;
    write_u16_le(buf, OFF_FLAGS, v)
}
pub fn reserved(buf: &[u8]) -> DbResult<u64> {
    validate(buf)?;
    read_u64_le(buf, OFF_RESERVED)
}
pub fn set_reserved(buf: &mut [u8], v: u64) -> DbResult<()> {
    validate(buf)?;
    write_u64_le(buf, OFF_RESERVED, v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::PAGE_SIZE;
    use crate::page::{SLOTTED_HEADER_SIZE, SLOTTED_SLOT_SIZE};

    /// Kiểm tra các invariant cơ bản của PageHeader.
    /// kiểm tra ở cấp struct (snapshot), chưa liên quan đến on-disk bytes.
    fn check_invariants(h: &PageHeader) {
        let header_size = SLOTTED_HEADER_SIZE as u16;
        let slot_size = SLOTTED_SLOT_SIZE as u16;
        let page_size = PAGE_SIZE as u16;

        // lower phải bắt đầu sau header
        assert!(h.lower() >= header_size, "lower must be >= HEADER_SIZE");

        // upper không được vượt quá kích thước page
        assert!(h.upper() <= page_size, "upper must be <= PAGE_SIZE");

        // lower luôn nằm trước hoặc bằng upper (free space = upper - lower)
        assert!(h.lower() <= h.upper(), "lower must be <= upper");

        let expected_lower = header_size + h.slot_count() * slot_size;
        assert_eq!(
            h.lower(),
            expected_lower,
            "lower must equal HEADER_SIZE + slot_count*SLOT_SIZE"
        );
    }

    #[test]
    fn test_getters() {
        let header_size = SLOTTED_HEADER_SIZE as u16;
        let page_size = PAGE_SIZE as u16;

        let h = PageHeader {
            lower: header_size,
            upper: page_size,
            slot_count: 0,
            flags: 0,
            reserved: 0,
        };

        assert_eq!(h.lower(), header_size);
        assert_eq!(h.upper(), page_size);
        assert_eq!(h.slot_count(), 0);
        assert_eq!(h.flags(), 0);
        assert_eq!(h.reserved(), 0);
    }

    #[test]
    fn test_flags_bits() {
        let header_size = SLOTTED_HEADER_SIZE as u16;
        let page_size = PAGE_SIZE as u16;

        // sample
        // - page_type = 2 (btree_internal) ở bits 0..3
        // - HAS_FREE_SLOTS ở bit 4
        let page_type: u16 = 2;
        let has_free_slots: u16 = 1 << 4;

        let h = PageHeader {
            lower: header_size,
            upper: page_size,
            slot_count: 0,
            flags: (page_type & 0x000F) | has_free_slots,
            reserved: 0,
        };

        let flags = h.flags();
        let extracted_page_type = flags & 0x000F;

        // Kiểm tra bit HAS_FREE_SLOTS
        let extracted_has_free = (flags & (1 << 4)) != 0;

        assert_eq!(extracted_page_type, 2);
        assert!(extracted_has_free);
    }

    #[test]
    fn test_invariants_ok() {
        let header_size = SLOTTED_HEADER_SIZE as u16;
        let slot_size = SLOTTED_SLOT_SIZE as u16;
        let page_size = PAGE_SIZE as u16;

        // valid header theo công thức lower = HEADER + slot_count*SLOT_SIZE
        let slot_count: u16 = 10;
        let lower = header_size + slot_count * slot_size;

        let h = PageHeader {
            lower,
            upper: page_size,
            slot_count,
            flags: 0,
            reserved: 0,
        };

        check_invariants(&h);
    }

    #[test]
    #[should_panic]
    fn test_invariants_fail_lower_formula() {
        let header_size = SLOTTED_HEADER_SIZE as u16;
        let page_size = PAGE_SIZE as u16;

        // header sai cần đảm bảo invariant sẽ "bắt lỗi"
        let h = PageHeader {
            lower: header_size + 1, // sai công thức
            upper: page_size,
            slot_count: 0,
            flags: 0,
            reserved: 0,
        };

        check_invariants(&h);
    }

    #[test]
    fn test_struct_size_sanity() {
        // sanity check: struct có đúng 16 bytes trong build hiện tại.
        // KHÔNG được dựa vào layout struct để serialize trực tiếp ra file.
        assert_eq!(std::mem::size_of::<PageHeader>(), 16);
    }

    // public helper -----------
    fn new_page_buf() -> Vec<u8> {
        vec![0u8; PAGE_SIZE]
    }

    #[test]
    fn test_validate_too_small() {
        let buf = [0u8; 15];
        assert!(validate(&buf).is_err());
    }

    #[test]
    fn test_validate_ok_min_size() {
        let buf = new_page_buf();
        assert!(validate(&buf).is_ok());
    }

    #[test]
    fn test_init_empty_sets_fields() {
        let mut buf = new_page_buf();

        let page_type: u16 = 2;
        init_empty(&mut buf, page_type).unwrap();

        assert_eq!(lower(&buf).unwrap(), SLOTTED_HEADER_SIZE as u16);
        assert_eq!(upper(&buf).unwrap(), PAGE_SIZE as u16);
        assert_eq!(slot_count(&buf).unwrap(), 0);
        assert_eq!(flags(&buf).unwrap(), page_type & 0x000F);
        assert_eq!(reserved(&buf).unwrap(), 0);
    }

    #[test]
    fn test_header_setters_roundtrip() {
        let mut buf = new_page_buf();
        init_empty(&mut buf, 0).unwrap();

        set_lower(&mut buf, 123).unwrap();
        assert_eq!(lower(&buf).unwrap(), 123);

        set_upper(&mut buf, 4000).unwrap();
        assert_eq!(upper(&buf).unwrap(), 4000);

        set_slot_count(&mut buf, 10).unwrap();
        assert_eq!(slot_count(&buf).unwrap(), 10);

        set_flags(&mut buf, 0x00F2).unwrap();
        assert_eq!(flags(&buf).unwrap(), 0x00F2);

        set_reserved(&mut buf, 0x1122_3344_5566_7788).unwrap();
        assert_eq!(reserved(&buf).unwrap(), 0x1122_3344_5566_7788);
    }
}
