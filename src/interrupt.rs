use instructions::push_op;
use mmu::{IE_REG, IF_REG, MMU};

pub const IF_VBLANK_BIT: u8 = 1;
pub const IF_LCDC_BIT: u8 = 2;
pub const IF_TMR_BIT: u8 = 4;
pub const IF_SERIAL_BIT: u8 = 8;
pub const IF_INP_BIT: u8 = 16;

// Address of interrupt handlers
pub const VBLANK_ADDR: u16 = 0x40;
pub const LCDC_ADDR: u16 = 0x48;
pub const TMR_ADDR: u16 = 0x50;
pub const SERIAL_ADDR: u16 = 0x58;
pub const INP_ADDR: u16 = 0x60;

fn interrupt(mmu: &mut MMU, bit: u8, addr: u16) {
    println!(
        "INTERRUPT! bit {}, addr 0x{:04X}, IE: {:02X}, halted: {}",
        bit,
        addr,
        mmu.direct_read(IE_REG),
        mmu.reg.halted
    );

    mmu.wakeup_if_halted();
    mmu.clear_if_reg_bits(bit);
    let pc = mmu.reg.pc;
    push_op(mmu, pc);
    mmu.reg.pc = addr;
    mmu.reg.ime = 0;
}

pub fn handle_interrupts(mmu: &mut MMU) {
    if mmu.reg.ime == 1 {
        mmu.reg.ime = 2;
        return;
    }

    if mmu.reg.ime == 2 {
        let if_reg = mmu.direct_read(IF_REG);
        let ie_reg = mmu.direct_read(IE_REG);
        let masked = if_reg & ie_reg;

        if masked & IF_VBLANK_BIT != 0 {
            interrupt(mmu, IF_VBLANK_BIT, VBLANK_ADDR);
        } else if masked & IF_LCDC_BIT != 0 {
            interrupt(mmu, IF_LCDC_BIT, LCDC_ADDR)
        } else if masked & IF_TMR_BIT != 0 {
            interrupt(mmu, IF_TMR_BIT, TMR_ADDR)
        } else if masked & IF_SERIAL_BIT != 0 {
            interrupt(mmu, IF_SERIAL_BIT, SERIAL_ADDR)
        } else if masked & IF_INP_BIT != 0 {
            interrupt(mmu, IF_INP_BIT, INP_ADDR)
        }
    }
}
