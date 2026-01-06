use crate::{DbError, DbResult};

/// Real DB cần:
/// detect corruption
/// return error có ý nghĩa
/// không crash vì dữ liệu sai
///
/// Trong DB engine, data corrupt hoặc bug offset rất dễ xảy ra.
/// Nếu panic -> crash mà không rõ vì sao.
/// Nếu trả error có context (off/size/len) -> debug fast
/// Giúp sau check invariant checks: "page corrupt".
#[inline]
fn checked_range(len: usize, off: usize, size: usize) -> DbResult<std::ops::Range<usize>> {
    if off > len || size > len || off + size > len {
        return Err(DbError::OutOfBounds { off, size, len });
    }
    Ok(off..off + size)
}

#[inline]
pub fn read_u16_le(buf: &[u8], off: usize) -> DbResult<u16> {
    let r = checked_range(buf.len(), off, 2)?;
    Ok(u16::from_le_bytes([buf[r.start], buf[r.start + 1]]))
}

#[inline]
pub fn write_u16_le(buf: &mut [u8], off: usize, v: u16) -> DbResult<()> {
    let r = checked_range(buf.len(), off, 2)?;
    let b = v.to_le_bytes();
    buf[r.start] = b[0];
    buf[r.start + 1] = b[1];
    Ok(())
}

#[inline]
pub fn read_u32_le(buf: &[u8], off: usize) -> DbResult<u32> {
    // Encoding style: explicit bounds check + fixed-size byte array conversion.
    // Alternatives (not used now for newbie)
    // 1) let bytes: [u8; 4] = buf[r].try_into().unwrap();
    // 2) buf[r].copy_from_slice(&v.to_le_bytes());

    let r = checked_range(buf.len(), off, 4)?;
    Ok(u32::from_le_bytes([
        buf[r.start],
        buf[r.start + 1],
        buf[r.start + 2],
        buf[r.start + 3],
    ]))
}

#[inline]
pub fn write_u32_le(buf: &mut [u8], off: usize, v: u32) -> DbResult<()> {
    let r = checked_range(buf.len(), off, 4)?;
    let b = v.to_le_bytes();
    buf[r.start] = b[0];
    buf[r.start + 1] = b[1];
    buf[r.start + 2] = b[2];
    buf[r.start + 3] = b[3];
    Ok(())
}

#[inline]
pub fn read_u64_le(buf: &[u8], off: usize) -> DbResult<u64> {
    let r = checked_range(buf.len(), off, 8)?;
    Ok(u64::from_le_bytes([
        buf[r.start],
        buf[r.start + 1],
        buf[r.start + 2],
        buf[r.start + 3],
        buf[r.start + 4],
        buf[r.start + 5],
        buf[r.start + 6],
        buf[r.start + 7],
    ]))
}

#[inline]
pub fn write_u64_le(buf: &mut [u8], off: usize, v: u64) -> DbResult<()> {
    let r = checked_range(buf.len(), off, 8)?;
    let b = v.to_le_bytes();
    buf[r.start] = b[0];
    buf[r.start + 1] = b[1];
    buf[r.start + 2] = b[2];
    buf[r.start + 3] = b[3];
    buf[r.start + 4] = b[4];
    buf[r.start + 5] = b[5];
    buf[r.start + 6] = b[6];
    buf[r.start + 7] = b[7];
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_u32() {
        let mut buf = [0u8; 16];
        write_u32_le(&mut buf, 4, 0x1122_3344).unwrap();
        let v = read_u32_le(&buf, 4).unwrap();
        assert_eq!(v, 0x1122_3344);
    }

    #[test]
    fn test_read_write_u64() {
        let mut buf = [0u8; 32];
        write_u64_le(&mut buf, 8, 0x1122_3344_5566_7788).unwrap();
        let v = read_u64_le(&buf, 8).unwrap();
        assert_eq!(v, 0x1122_3344_5566_7788);
    }

    #[test]
    fn test_out_of_bounds() {
        let mut buf = [0u8; 8];
        let err = write_u64_le(&mut buf, 4, 1).unwrap_err();
        match err {
            crate::error::DbError::OutOfBounds { .. } => {}
            _ => panic!("expected OutOfBounds"),
        }
    }
}
