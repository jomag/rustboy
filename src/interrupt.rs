
use memory::{ IF_REG, IE_REG };
use cpu::Cpu;
use instructions::push_op;

pub const VBLANK_BIT: u8 = 1;
pub const LCDC_BIT: u8 = 2;
pub const TMR_BIT: u8 = 4;
pub const SERIAL_BIT: u8 = 8;
pub const INP_BIT: u8 = 16;

// Address of interrupt handlers
pub const VBLANK_ADDR: u16 = 0x40;
pub const LCDC_ADDR: u16 = 0x48;
pub const TMR_ADDR: u16 = 0x50;
pub const SERIAL_ADDR: u16 = 0x58;
pub const INP_ADDR: u16 = 0x60;

fn interrupt(cpu: &mut Cpu, bit: u8, addr: u16) {
    println!("INTERRUPT! bit {}, addr 0x{:04X}", bit, addr);
    cpu.mem.mem[IF_REG as usize] &= !bit;
    let pc = cpu.reg.pc;
    push_op(cpu, pc);
    cpu.reg.pc = addr;
    cpu.reg.ime = 0;
}

pub fn handle_interrupts(cpu: &mut Cpu) {
    if cpu.reg.ime == 1 {
        cpu.reg.ime = 2;
        return;
    }

    if cpu.reg.ime == 2 {
        let if_reg = cpu.mem.mem[IF_REG as usize];
        let ie_reg = cpu.mem.mem[IE_REG as usize];
        let masked = if_reg & ie_reg;

        if masked & VBLANK_BIT != 0 {
            interrupt(cpu, VBLANK_BIT, VBLANK_ADDR);
        } else if masked & LCDC_BIT != 0 {
            interrupt(cpu, LCDC_BIT, LCDC_ADDR)
        } else if masked & TMR_BIT != 0 {
            interrupt(cpu, TMR_BIT, TMR_ADDR)
        } else if masked & SERIAL_BIT != 0 {
            interrupt(cpu, SERIAL_BIT, SERIAL_ADDR)
        } else if masked & INP_BIT != 0 {
            interrupt(cpu, INP_BIT, INP_ADDR)
        }
    }
}
