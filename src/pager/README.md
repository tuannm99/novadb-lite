# pager/

`pager/` là tầng **quản lý page I/O** và **không gian lưu trữ vật lý** cho database.
Pager không biết SQL, không biết B-Tree/Heap logic; chỉ biết:

- đọc/ghi page theo `page_id`
- cấp phát / thu hồi page
- quản lý metadata tối thiểu (free list, next id, v.v.)
- đảm bảo write đúng cách (về sau mới thêm WAL/flush policy)

> Mục tiêu: mọi access method (Heap, B-Tree clustered, Overflow, …) đều dùng chung `pager/`.

---

## Core API

### PageId

- `type PageId = u32;`

### Read/Write contract

- Page size cố định: `PAGE_SIZE` bytes.
- `read_page(pid)` luôn trả đúng `PAGE_SIZE`.
- `write_page(pid, buf)` yêu cầu `buf.len() == PAGE_SIZE`.

### Allocation

- `alloc_page()` trả về `PageId` mới hoặc reuse từ free list.
- `free_page(pid)` đưa page vào free list (không xoá data ngay).
- Phase đầu có thể dùng in-memory free list; sau này persist.

---

## Page types / invariants

- Page bytes luôn được validate ở 2 tầng:
  1. Pager validate "physical" (đúng size, đúng offset)
  2. `page/` validate "logical" (header/slot invariants)

Pager không cần hiểu `page_type`, nhưng vẫn có thể hỗ trợ debug:

- allow đọc `flags/page_type` ở header để log.

---

## Minimal implementation plan

### Phase 0 (đủ dùng cho slotted page + btree leaf)

- [ ] `FilePager::open(path)`
- [ ] `read_page(pid) -> [u8; PAGE_SIZE]` (hoặc Vec<u8>)
- [ ] `write_page(pid, &[u8])`
- [ ] `alloc_page()`: append file (pid = file_len / PAGE_SIZE)
- [ ] `free_page(pid)`: in-memory free list (Vec<PageId>)

### Phase 1 (ổn định hơn)

- [ ] persist free list (meta page 0)
- [ ] bounds check: pid không vượt file len
- [ ] option: `zero_on_alloc` / `zero_on_free`

### Phase 2 (durability)

- [ ] flush policy
- [ ] WAL + lsn in page header
- [ ] checksum

---

## Testing approach

### Unit tests (pager only)

- `alloc_page` tăng pid đúng
- `write_page` rồi `read_page` roundtrip
- `free_page` rồi `alloc_page` reuse pid
- pid out of range => error (English messages)

### Integration tests (pager + page/)

- alloc page -> init slotted -> insert/get/delete -> write -> read lại -> validate ok

> - `cargo test -- --nocapture`
> - `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

---

## Notes

`pager/` chính là "segment/physical heap" ở tầng thấp: container của pages.

- `page/` slotted layout, pager chỉ việc cấp phát/đọc/ghi page.
