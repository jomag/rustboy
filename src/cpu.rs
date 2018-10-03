
use registers::Registers;
use memory::Memory;
use timer::Timer;
use interrupt::handle_interrupts;
use instructions;
use lcd::LCD;

pub struct Cpu {
    pub reg: Registers,
    pub mem: Memory,
    pub lcd: LCD,

    // Temporary placement of display status
    pub display_updated: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: Registers::new(),
            mem: Memory::new(),
            lcd: LCD::new(),
            display_updated: false
        }
    }

    pub fn exec_op(&mut self) {
        instructions::step(self);
        handle_interrupts(self);
    }

    pub fn tick(&mut self, cycles: u32) {
        self.mem.timer.update(cycles);
        if self.lcd.update(cycles, &mut self.mem) {
            self.display_updated = true;
        }
    }

    pub fn fetch(&mut self) -> u8 {
        let pc = self.reg.pc;
        let value = self.mem.read(pc);
        self.reg.pc = pc.wrapping_add(1);
        self.tick(4);
        value
    }

    pub fn fetch_u16(&mut self) -> u16 {
        let lo = self.fetch();
        let hi = self.fetch();
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let value = self.mem.read(addr);
        self.tick(4);
        value
    }

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.mem.write(addr, value);
        self.tick(4);
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        self.mem.write(addr, (value & 0xFF) as u8);
        self.mem.write(addr + 1, (value >> 8) as u8);
        self.tick(8);
    }

    pub fn pop(&mut self) -> u16 {
        let sp = self.reg.sp;
        let v = self.read_u16(sp);
        self.reg.sp = sp.wrapping_add(2);
        v
    }
}
