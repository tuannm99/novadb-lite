package novadblite

const (
	slotDead       uint16 = 1 << 0
	slotRedirected uint16 = 1 << 1
	slotOverflow   uint16 = 1 << 2
)

const (
	offSlotOffset = 0
	offSlotLen    = 2
	offSlotFlags  = 4
)

type Slot struct {
	offset uint16
	length uint16
	flags  uint16
}

func newSlot(offset, length, flags uint16) Slot {
	return Slot{offset: offset, length: length, flags: flags}
}

func (s Slot) Offset() uint16 {
	return s.offset
}

func (s Slot) Len() uint16 {
	return s.length
}

func (s Slot) Flags() uint16 {
	return s.flags
}

func (s *Slot) MarkDead() {
	s.flags |= slotDead
}

func (s *Slot) MarkRedirected() {
	s.flags |= slotRedirected
}

func (s *Slot) MarkOverflow() {
	s.flags |= slotOverflow
}

func slotOff(slotID uint16) int {
	return SlottedHeaderSize + int(slotID)*SlottedSlotSize
}

func currentPos(buf []byte, slotID uint16) (int, error) {
	base := slotOff(slotID)
	if base+SlottedSlotSize > len(buf) {
		return 0, newCorruption("slot entry out of bounds")
	}
	return base, nil
}

func readSlot(buf []byte, slotID uint16) (Slot, error) {
	pos, err := currentPos(buf, slotID)
	if err != nil {
		return Slot{}, err
	}
	offset, err := readU16LE(buf, pos+offSlotOffset)
	if err != nil {
		return Slot{}, err
	}
	length, err := readU16LE(buf, pos+offSlotLen)
	if err != nil {
		return Slot{}, err
	}
	flags, err := readU16LE(buf, pos+offSlotFlags)
	if err != nil {
		return Slot{}, err
	}
	return Slot{offset: offset, length: length, flags: flags}, nil
}

func writeSlot(buf []byte, slotID uint16, slot Slot) error {
	pos, err := currentPos(buf, slotID)
	if err != nil {
		return err
	}
	if err := writeU16LE(buf, pos+offSlotOffset, slot.offset); err != nil {
		return err
	}
	if err := writeU16LE(buf, pos+offSlotLen, slot.length); err != nil {
		return err
	}
	if err := writeU16LE(buf, pos+offSlotFlags, slot.flags); err != nil {
		return err
	}
	return nil
}

func isDead(flags uint16) bool {
	return flags&slotDead != 0
}

func isRedirected(flags uint16) bool {
	return flags&slotRedirected != 0
}

func isOverflow(flags uint16) bool {
	return flags&slotOverflow != 0
}
