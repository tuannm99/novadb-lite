use crate::DbResult;

use crate::page::header;
use crate::page::slot::{is_dead, read_slot, slot_off, write_slot, Slot};

/// SlottedPage là API cấp cao thao tác trên 1 page bytes theo layout slotted-page.
/// - Header ở đầu page (fixed 16 bytes)
/// - Slot directory grow từ thấp lên (lower tăng dần)
/// - Tuple/data grow từ cao xuống (upper giảm dần)
pub struct SlottedPage<'a> {
    buf: &'a mut [u8],
}

impl<'a> SlottedPage<'a> {
    /// Tạo wrapper trên buffer page.
    /// Lưu ý: không init header ở đây, chỉ wrap.
    pub fn new(buf: &'a mut [u8]) -> DbResult<Self> {
        // nên check buf.len() >= PAGE_SIZE hoặc ít nhất >= SLOTTED_HEADER_SIZE
        todo!()
    }

    /// Khởi tạo page rỗng.
    /// - lower = HEADER_SIZE
    /// - upper = PAGE_SIZE
    /// - slot_count = 0
    /// - flags = page_type (bits 0..3)
    pub fn init(&mut self, page_type: u16) -> DbResult<()> {
        todo!()
    }

    /// Validate page header + basic invariants.
    pub fn validate(&self) -> DbResult<()> {
        todo!()
    }

    /// Số slot đã cấp phát (không giảm).
    pub fn slot_count(&self) -> DbResult<u16> {
        todo!()
    }

    /// Free space hiện tại trong page (upper - lower).
    pub fn free_space(&self) -> DbResult<u16> {
        todo!()
    }

    /// Insert record bytes vào page.
    /// Ý tưởng thuật toán:
    /// 1) Đọc lower/upper/slot_count.
    /// 2) Tìm slot tombstone để reuse (nếu muốn reuse), hoặc cấp slot_id mới.
    /// 3) Tính upper_new = upper - data.len()
    /// 4) Check đủ chỗ:
    ///    - Nếu cấp slot mới: cần thêm SLOT_SIZE bytes cho slot directory (lower tăng)
    ///    - Nếu reuse slot: không tăng lower
    /// 5) Copy data vào vùng [upper_new..upper)
    /// 6) Ghi slot entry: offset=upper_new, len=data.len, flags=0
    /// 7) Update header: upper=upper_new, lower/slot_count nếu slot mới
    pub fn insert(&mut self, data: &[u8]) -> DbResult<u16> {
        todo!()
    }

    /// Lấy record bytes theo slot_id.
    /// Trả None nếu slot DEAD.
    /// Các check cần có:
    /// - slot_id < slot_count
    /// - slot.offset + slot.len <= PAGE_SIZE
    pub fn get<'b>(&'b self, slot_id: u16) -> DbResult<Option<&'b [u8]>> {
        todo!()
    }

    /// Delete slot_id: set flag DEAD, không reclaim data ngay (tombstone).
    /// Nếu bạn muốn reuse slot:
    /// - set page header flag HAS_FREE_SLOTS (bit 4)
    pub fn delete(&mut self, slot_id: u16) -> DbResult<()> {
        todo!()
    }

    /// (Optional) Tìm slot tombstone để reuse.
    /// Nếu page header có HAS_FREE_SLOTS thì scan slot directory, return slot_id đầu tiên DEAD.
    fn find_free_slot(&self) -> DbResult<Option<u16>> {
        todo!()
    }
}
