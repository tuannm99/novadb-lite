# [#2] sqlite

## Scopes

- Single file storage, limited to 5Gb.
- SlottedPage Layout
- Simple CRUD operations on TABLE
- Index-Organize Storage (Cluster idx - btree int only)
- OverflowPage
- Simple Buffer
- Global lock - database level lock only -- can extends if can
- Wal for support durability, simple transaction
- Vietnamese comment

## How to write Rust code (Conventions)

Mục tiêu của repo này là build một DB engine kiểu “SQLite-lite” bằng Rust (Newbie). Code ưu tiên **đọc được**, **đúng invariants**, và **dễ test** hơn là tối ưu sớm.

### General rules

- **Comment:** viết tiếng Việt.
- **Error / assert / log strings:** viết tiếng Anh (dễ search, tooling friendly).
- **No magic numbers:** dùng `const` cho offsets/sizes/bitmasks.
- **One source of truth:** constants không duplicate ở nhiều nơi (tránh lệch).
- **Prefer small functions:** mỗi hàm làm 1 việc, dễ test.

### Module layering (NOW)

- `page/raw.rs`
  - Low-level read/write primitives (little-endian, bounds check).
  - Không chứa logic DB.
- `page/header.rs`
  - `header::*` là các **free functions** đọc/ghi header trực tiếp trên `&[u8]` / `&mut [u8]`.
  - Không giữ `&mut [u8]` lâu để tránh borrow issues.
- `page/slotted.rs`
  - Slot entry layout + read/write slot entry.
- `page/slotted_page.rs` (hoặc `page/slotted/mod.rs`)
  - `SlottedPage<'a>` là API cấp cao (`insert/get/delete`) gọi `header::*` và `slotted::*`.

> Lý do dùng free functions cho `header/slot`: tránh kẹt borrow checker khi vừa sửa header vừa slice data vùng khác trong cùng page.

### Rust style

- Use `Result<T, E>` alias (`DbResult<T>`) cho mọi hàm có thể fail.
- Dùng `?` để propagate error thay vì `match` dài (khi hợp lý).
- Dùng `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` cho các struct snapshot nhỏ (header, slot).
- Ưu tiên `usize` cho indexing/slicing; dùng `u16/u32/u64` cho on-disk fields.

### Types guideline (important)

- `usize`: dùng cho index/offset trong memory (`buf.len()`, `buf[a..b]`).
- `u16`: dùng cho field trong page (offset/len/lower/upper) vì page size nhỏ.
- Mọi cast từ `usize -> u16` phải có check trước (không overflow).

### Error handling

- Tất cả error messages **English**.
- Khi validate bounds:
  - luôn check `off + needed <= buf.len()`
  - nếu fail, return `Err(...)` (message EN)

### Formatting / lint

- Run:
  - `cargo fmt`
  - `cargo test`
- Prefer `rustfmt` defaults (repo đã có `rustfmt.toml`).

### Commit discipline (optional but helpful)

- Commit nhỏ theo từng bước:
  1. raw read/write
  2. header funcs
  3. slot funcs
  4. slotted page get/delete
  5. slotted page insert
  6. overflow page
  7. btree, wal, buffer...
