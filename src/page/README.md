# Storage Layout

## Page Layout

```text
|<------------------------- PAGE_SIZE -------------------------->|
+----------------------------------------------------------------+
| Page Header (16 bytes)                                         |
|                                                                |
|  lower (u16)  | upper (u16) | slot_count (u16) | flags (u16)   |
|  reserved (u64)                                                |
+----------------------------------------------------------------+
| Slot Directory (grows UP ->)                                   |
|                                                                |
|  slot[0] | slot[1] | slot[2] | ...                             |
|                                                                |
|  lower = HEADER + slot_count * SLOT_SIZE                       |
+----------------------------------------------------------------+
|                         FREE SPACE                             |
|                   (upper - lower bytes)                        |
+----------------------------------------------------------------+
| Tuple / Record Data (grows DOWN <- )                           |
|                                                                |
|  data[n] | data[n-1] | ... | data[0]                           |
|                                                                |
|  upper                                                PAGE_SIZE|
+----------------------------------------------------------------+
```

---

### 🧠 ZOOM-IN: Page Header (16 bytes)

```text
byte offset
0      2      4      6      8             16
+------+------+------+------+----------------+
|lower |upper |slots |flags |   reserved     |
+------+------+------+------+----------------+

```

### FLAGS FIELD (u16)

```text

bit index:  15 ............ 8 7 6 5 4 3 2 1 0
            [   future     ] R C Z F P P P P
                               ^ ^ ^ ^
                               | | | |
                               | | | +-- page type (0..3)
                               | | +---- HAS_FREE_SLOTS
                               | +------ IS_COMPRESSED
                               +-------- IS_CHECKSUMMED

```

### Bits operation

```rs

/// << – tạo flag
let FLAG_HAS_FREE_SLOTS = 1 << 4; // 0001 0000
// ->
// đánh dấu 1 bit tại vị trí 4

/// | - Bật bit flags
let flags = flags | FLAG_HAS_FREE_SLOTS;
// Quy tắc:
// 1 | X = 1 → bật
// 0 | X = X → giữ nguyên
// -> KHÔNG phá bit khác

/// & ! – TẮT bit (clear flag)
let flags = flags & !FLAG_HAS_FREE_SLOTS;
// ->
// !FLAG → mask toàn 1 trừ bit cần clear
// & với mask → bit đó về 0, bit khác giữ nguyên

/// & mask – CHECK bit
(flags & FLAG_HAS_FREE_SLOTS) != 0
// ->
// Bit = 1 → khác 0
// Bit = 0 → bằng 0

/// Mask low bits (page_type)
flags & 0x000F
// 0x000F = 0000 0000 0000 1111
// Chỉ giữ 4 bit thấp
// Flag khác không bị ảnh hưởng


/// Set page_type không phá flag
flags = (flags & !0x000F) | PAGE_TYPE_HEAP;
// -> 2 bước
// flags & !0x000F → xoá page_type cũ
// | PAGE_TYPE_* → set page_type mới

```

- defined

```text

// low 4 bits
flags & 0x000F        // page type

// bit 4
FLAG_HAS_FREE_SLOTS = 1 << 4

// bit 5
FLAG_IS_COMPRESSED  = 1 << 5

// bit 6
FLAG_IS_CHECKSUMMED = 1 << 6
```

- add flags

```text
flags = flags | FLAG_HAS_FREE_SLOTS;

# before: 0000 0000 0000 0010
# after : 0000 0000 0001 0010
#                      ^
#                      HAS_FREE_SLOTS -> Yes
```

- clear flags

```text
flags = flags & !FLAG_HAS_FREE_SLOTS;

# before: 0000 0000 0001 0010
# after : 0000 0000 0000 0010
#                      ^
#                      HAS_FREE_SLOTS -> No
```

- check page type

```text
(flags & 0x000F) == PAGE_TYPE_BTREE_LEAF
```

- Set page type (KHÔNG phá flag khác)

```text
flags = (flags & !0x000F) | PAGE_TYPE_HEAP;
```

### Slot (BYTES)

```text
Slot entry (6 bytes, little-endian):
+--------+--------+--------+
| offset | length | flags  |
+--------+--------+--------+
  u16       u16      u16

page_type (low 4 bits):
0=heap, 1=btree_leaf, 2=btree_internal, 3=overflow
slot.flags cũng là bitmask
slot::is_dead(flags) → (flags & SLOT_FLAG_DEAD) != 0
```
