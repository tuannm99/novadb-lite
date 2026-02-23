package novadblite

type SlottedPage struct {
	buf []byte
}

func NewSlottedPage(buf []byte) (*SlottedPage, error) {
	if len(buf) != PageSize {
		return nil, newCorruption("buffer length must equal PAGE_SIZE")
	}
	return &SlottedPage{buf: buf}, nil
}

func (p *SlottedPage) Init(pageType uint16) (*SlottedPage, error) {
	if err := initEmptyHeader(p.buf, pageType); err != nil {
		return nil, err
	}
	return p, nil
}

func (p *SlottedPage) ValidateFull() error {
	if err := p.ValidateHeader(); err != nil {
		return err
	}
	up, err := upper(p.buf)
	if err != nil {
		return err
	}
	sc, err := slotCount(p.buf)
	if err != nil {
		return err
	}
	for i := uint16(0); i < sc; i++ {
		s, err := readSlot(p.buf, i)
		if err != nil {
			return err
		}
		if !isDead(s.Flags()) {
			start := int(s.Offset())
			length := int(s.Len())
			end := start + length
			if end < start {
				return newCorruption("tuple end overflow")
			}
			if end > PageSize {
				return newCorruption("corrupt slot: tuple out of bounds")
			}
			if start < int(up) {
				return newCorruption("corrupt slot: tuple overlaps free space")
			}
		}
	}
	return nil
}

func (p *SlottedPage) ValidateHeader() error {
	lo, err := lower(p.buf)
	if err != nil {
		return err
	}
	up, err := upper(p.buf)
	if err != nil {
		return err
	}
	sc, err := slotCount(p.buf)
	if err != nil {
		return err
	}

	if int(lo) < SlottedHeaderSize {
		return newCorruption("corrupt header: lower < header size")
	}
	if int(up) > PageSize {
		return newCorruption("corrupt header: upper > PAGE_SIZE")
	}
	if lo > up {
		return newCorruption("corrupt header: lower > upper")
	}

	if int(sc) > (1<<31-1)/SlottedSlotSize {
		return newCorruption("corrupt header: slot_count overflow")
	}
	slotBytes := int(sc) * SlottedSlotSize
	expectedLo := SlottedHeaderSize + slotBytes
	if expectedLo > PageSize {
		return newCorruption("corrupt header: slot directory out of page")
	}
	if int(lo) != expectedLo {
		return newCorruption("corrupt header: lower != header_size + slot_count*slot_size")
	}
	return nil
}

func (p *SlottedPage) FreeSpace() (uint16, error) {
	up, err := upper(p.buf)
	if err != nil {
		return 0, err
	}
	lo, err := lower(p.buf)
	if err != nil {
		return 0, err
	}
	if up < lo {
		return 0, newCorruption("corrupt header: lower > upper")
	}
	return up - lo, nil
}

func (p *SlottedPage) Get(slotID uint16) ([]byte, bool, error) {
	if err := p.ValidateHeader(); err != nil {
		return nil, false, err
	}
	sc, err := slotCount(p.buf)
	if err != nil {
		return nil, false, err
	}
	if slotID >= sc {
		return nil, false, newInvalidArgument("invalid slot_id")
	}
	s, err := readSlot(p.buf, slotID)
	if err != nil {
		return nil, false, err
	}
	if isDead(s.Flags()) {
		return nil, false, nil
	}
	start := int(s.Offset())
	up, err := upper(p.buf)
	if err != nil {
		return nil, false, err
	}
	if start < int(up) {
		return nil, false, newCorruption("tuple overlaps free space")
	}
	length := int(s.Len())
	end := start + length
	if end < start {
		return nil, false, newCorruption("tuple end overflow")
	}
	if end > PageSize {
		return nil, false, newCorruption("tuple end must be <= PAGE_SIZE")
	}
	return p.buf[start:end], true, nil
}

func (p *SlottedPage) Insert(data []byte) (uint16, error) {
	if err := p.ValidateHeader(); err != nil {
		return 0, err
	}
	up, err := upper(p.buf)
	if err != nil {
		return 0, err
	}
	sc, err := slotCount(p.buf)
	if err != nil {
		return 0, err
	}
	if len(data) > int(^uint16(0)) {
		return 0, newCorruption("record is too large")
	}
	needDataLen := uint16(len(data))

	reuseID, err := p.findFreeSlot()
	if err != nil {
		return 0, err
	}
	canReuse := reuseID != nil

	slotID := sc
	if canReuse {
		slotID = *reuseID
	}
	var needSlot uint16
	if !canReuse {
		needSlot = uint16(SlottedSlotSize)
	}

	needTotal := int(needDataLen) + int(needSlot)
	free, err := p.FreeSpace()
	if err != nil {
		return 0, err
	}
	if needTotal > int(free) {
		return 0, newNoSpace("not enough space")
	}

	if up < needDataLen {
		return 0, newCorruption("record is too large")
	}
	upperNew := up - needDataLen
	copy(p.buf[int(upperNew):int(up)], data)

	if err := writeSlot(p.buf, slotID, newSlot(upperNew, needDataLen, 0)); err != nil {
		return 0, err
	}
	if !canReuse {
		if err := setSlotCount(p.buf, sc+1); err != nil {
			return 0, err
		}
		lowerNew := uint16(SlottedHeaderSize) + (sc+1)*uint16(SlottedSlotSize)
		if err := setLower(p.buf, lowerNew); err != nil {
			return 0, err
		}
	}
	if err := setUpper(p.buf, upperNew); err != nil {
		return 0, err
	}
	return slotID, nil
}

func (p *SlottedPage) Update(slotID uint16, data []byte) (bool, error) {
	if err := p.ValidateHeader(); err != nil {
		return false, err
	}
	sc, err := slotCount(p.buf)
	if err != nil {
		return false, err
	}
	if slotID >= sc {
		return false, newInvalidArgument("invalid slot_id")
	}
	s, err := readSlot(p.buf, slotID)
	if err != nil {
		return false, err
	}
	if isDead(s.Flags()) {
		return false, newCorruption("slot is dead")
	}
	if len(data) > int(^uint16(0)) {
		return false, newCorruption("record is too large")
	}
	need := uint16(len(data))
	oldLen := s.Len()

	if need <= oldLen {
		start := int(s.Offset())
		endNew := start + int(need)
		endOld := start + int(oldLen)
		copy(p.buf[start:endNew], data)
		for i := endNew; i < endOld; i++ {
			p.buf[i] = 0
		}
		if err := writeSlot(p.buf, slotID, newSlot(s.Offset(), need, s.Flags())); err != nil {
			return false, err
		}
		return false, nil
	}

	free, err := p.FreeSpace()
	if err != nil {
		return false, err
	}
	if need > free {
		return false, newNoSpace("not enough space")
	}
	up, err := upper(p.buf)
	if err != nil {
		return false, err
	}
	if up < need {
		return false, newCorruption("record is too large")
	}
	upperNew := up - need
	copy(p.buf[int(upperNew):int(up)], data)
	if err := writeSlot(p.buf, slotID, newSlot(upperNew, need, s.Flags())); err != nil {
		return false, err
	}
	if err := setUpper(p.buf, upperNew); err != nil {
		return false, err
	}
	return true, nil
}

func (p *SlottedPage) Delete(slotID uint16) error {
	if err := p.ValidateHeader(); err != nil {
		return err
	}
	sc, err := slotCount(p.buf)
	if err != nil {
		return err
	}
	if slotID >= sc {
		return newInvalidArgument("invalid slot_id")
	}
	s, err := readSlot(p.buf, slotID)
	if err != nil {
		return err
	}
	if isDead(s.Flags()) {
		return nil
	}
	s.MarkDead()
	if err := writeSlot(p.buf, slotID, s); err != nil {
		return err
	}
	pageFlags, err := flags(p.buf)
	if err != nil {
		return err
	}
	newFlags := setFlag(pageFlags, FlagHasFreeSlots)
	return setFlags(p.buf, newFlags)
}

func (p *SlottedPage) findFreeSlot() (*uint16, error) {
	pageFlags, err := flags(p.buf)
	if err != nil {
		return nil, err
	}
	if (pageFlags & FlagHasFreeSlots) == 0 {
		return nil, nil
	}
	sc, err := slotCount(p.buf)
	if err != nil {
		return nil, err
	}
	for i := uint16(0); i < sc; i++ {
		s, err := readSlot(p.buf, i)
		if err != nil {
			return nil, err
		}
		if isDead(s.Flags()) {
			return &i, nil
		}
	}
	newFlags := clearFlag(pageFlags, FlagHasFreeSlots)
	if err := setFlags(p.buf, newFlags); err != nil {
		return nil, err
	}
	return nil, nil
}
