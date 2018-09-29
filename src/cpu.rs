
use registers::Registers;
use timer::Timer;

struct Cpu {
    reg: Registers,
    timer: Timer
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: Registers::new()
            timer: Timer::new()
        }
    }

    pub fn run_cycle(&mut self) {
        self.timer.update(4)
    }

    pub fn fetch(&mut self) -> u8 {
        let pc = self.reg.pc;
        let v = self.mem.read(pc);
        self.reg.pc = pc + 1;
        self.run_cycle();
        v
    }

    pub fn step(&mut self) {
        let op = self.fetch();

        match op {
            // NOP:
            0x00 => {}
            0x01 => {
                let c = self.fetch()
                let b = self.fetch()
                reg.c = c;
                reg.b = b;
            }
        }
    }
}