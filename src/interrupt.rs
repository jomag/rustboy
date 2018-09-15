
use memory::{ Memory, IF_REG, IE_REG };
use registers::Registers;
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

fn interrupt(reg: &mut Registers, mem: &mut Memory, bit: u8, addr: u16) {
    println!("INTERRUPT! bit {}, addr 0x{:04X}", bit, addr);
    mem.mem[IF_REG as usize] &= !bit;
    let pc = reg.pc;
    push_op(reg, mem, pc);
    reg.pc = 0x40;
    reg.ime = false;
}

pub fn handle_interrupts(reg: &mut Registers, mem: &mut Memory) {
    if reg.ime {
        let if_reg = mem.mem[IF_REG as usize];
        let ie_reg = mem.mem[IE_REG as usize];
        let masked = if_reg & ie_reg;

        if masked & VBLANK_BIT != 0 {
            interrupt(reg, mem, VBLANK_BIT, VBLANK_ADDR);
        } else if masked & LCDC_BIT != 0 {
            interrupt(reg, mem, LCDC_BIT, LCDC_ADDR)
        } else if masked & TMR_BIT != 0 {
            interrupt(reg, mem, TMR_BIT, TMR_ADDR)
        } else if masked & SERIAL_BIT != 0 {
            interrupt(reg, mem, SERIAL_BIT, SERIAL_ADDR)
        } else if masked & INP_BIT != 0 {
            interrupt(reg, mem, INP_BIT, INP_ADDR)
        }
    }
}