use crate::error::{DbError, Result};

fn checked_range(length: usize, off: usize, size: usize) -> Result<(usize, usize)> {
    let end = off
        .checked_add(size)
        .ok_or_else(|| DbError::out_of_bounds(off, size, length))?;
    if off > length || size > length || end > length {
        return Err(DbError::out_of_bounds(off, size, length));
    }
    Ok((off, end))
}

pub(crate) fn read_u16_le(buf: &[u8], off: usize) -> Result<u16> {
    let (start, end) = checked_range(buf.len(), off, 2)?;
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&buf[start..end]);
    Ok(u16::from_le_bytes(bytes))
}

pub(crate) fn write_u16_le(buf: &mut [u8], off: usize, value: u16) -> Result<()> {
    let (start, end) = checked_range(buf.len(), off, 2)?;
    buf[start..end].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

pub(crate) fn read_u32_le(buf: &[u8], off: usize) -> Result<u32> {
    let (start, end) = checked_range(buf.len(), off, 4)?;
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&buf[start..end]);
    Ok(u32::from_le_bytes(bytes))
}

pub(crate) fn write_u32_le(buf: &mut [u8], off: usize, value: u32) -> Result<()> {
    let (start, end) = checked_range(buf.len(), off, 4)?;
    buf[start..end].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

pub(crate) fn read_u64_le(buf: &[u8], off: usize) -> Result<u64> {
    let (start, end) = checked_range(buf.len(), off, 8)?;
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[start..end]);
    Ok(u64::from_le_bytes(bytes))
}

pub(crate) fn write_u64_le(buf: &mut [u8], off: usize, value: u64) -> Result<()> {
    let (start, end) = checked_range(buf.len(), off, 8)?;
    buf[start..end].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn read_write_u32() {
        let mut buf = vec![0; 16];
        write_u32_le(&mut buf, 4, 0x11223344).unwrap();
        assert_eq!(0x11223344, read_u32_le(&buf, 4).unwrap());
    }

    #[test]
    fn read_write_u64() {
        let mut buf = vec![0; 32];
        write_u64_le(&mut buf, 8, 0x1122334455667788).unwrap();
        assert_eq!(0x1122334455667788, read_u64_le(&buf, 8).unwrap());
    }

    #[test]
    fn out_of_bounds() {
        let mut buf = vec![0; 8];
        let err = write_u64_le(&mut buf, 4, 1).unwrap_err();
        assert_eq!(ErrorKind::OutOfBounds, err.kind);
    }
}
