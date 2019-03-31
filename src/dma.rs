// A DMA transfer is initiated by writing a start address
// to memory position 0xFF46. The transfer will copy 160
// bytes from the start address to OAM memory.
//
// The start address is specified in increments of 0x100,
// starting at 0x8000.
//
// The transfer will begin 4 clock cycles after the write.
// During a transfer all reads of OAM memory will return 0xFF.

pub struct DMA {
    pub start_request: Option<u16>,
    pub start_request_delay: Option<u16>,
    pub start_address: Option<u16>,
    pub step: u16,
    pub oam: [u8; 0xA0],

    // This is to handle a quirk: the DMA address (0xFF46) should
    // always return the last written value on read operations,
    // or 0xFF it has not been written to. See Mooneye test "reg_read.gb"
    pub last_write_dma_reg: u8
}

impl DMA {
    pub fn new() -> Self {
        DMA {
            start_request: None,
            start_request_delay: None,
            start_address: None,
            step: 0,
            oam: [0; 0xA0],
            last_write_dma_reg: 0xFF
        }
    }

    pub fn is_active(&self) -> bool {
        self.start_address.is_some()
    }

    pub fn read(&self, address: u16) -> u8 {
        if self.is_active() {
            return 0xFF
        } else {
            return self.oam[address as usize];
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if !self.is_active() {
            self.oam[address as usize] = value;
        }
    }

    pub fn start(&mut self, start: u8) {
        self.start_request = Some((start as u16) << 8);
        self.last_write_dma_reg = start;
    }

    pub fn update(&mut self) {
        if self.start_request.is_some() {
            self.start_request_delay = self.start_request;
            self.start_request = None;
            return;
        }

        if self.start_request_delay.is_some() {
            self.start_address = self.start_request_delay;
            self.start_request_delay = None;
            self.step = 0;
            return
        }

        if self.start_address.is_some() {
            if self.step == 159 {
                self.start_address = None;
            } else {
                self.step += 1;
            }
        }
    }
}
