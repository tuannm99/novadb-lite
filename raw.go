package novadblite

import "encoding/binary"

func checkedRange(length, off, size int) (int, int, error) {
	if off > length || size > length || off+size > length {
		return 0, 0, newOutOfBounds(off, size, length)
	}
	return off, off + size, nil
}

func readU16LE(buf []byte, off int) (uint16, error) {
	start, end, err := checkedRange(len(buf), off, 2)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint16(buf[start:end]), nil
}

func writeU16LE(buf []byte, off int, v uint16) error {
	start, end, err := checkedRange(len(buf), off, 2)
	if err != nil {
		return err
	}
	binary.LittleEndian.PutUint16(buf[start:end], v)
	return nil
}

func readU32LE(buf []byte, off int) (uint32, error) {
	start, end, err := checkedRange(len(buf), off, 4)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint32(buf[start:end]), nil
}

func writeU32LE(buf []byte, off int, v uint32) error {
	start, end, err := checkedRange(len(buf), off, 4)
	if err != nil {
		return err
	}
	binary.LittleEndian.PutUint32(buf[start:end], v)
	return nil
}

func readU64LE(buf []byte, off int) (uint64, error) {
	start, end, err := checkedRange(len(buf), off, 8)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint64(buf[start:end]), nil
}

func writeU64LE(buf []byte, off int, v uint64) error {
	start, end, err := checkedRange(len(buf), off, 8)
	if err != nil {
		return err
	}
	binary.LittleEndian.PutUint64(buf[start:end], v)
	return nil
}
