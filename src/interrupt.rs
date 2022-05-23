use crate::instructions::push_op;
use crate::mmu::{IE_REG, IF_REG, MMU};

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
    /*
    println!(
        "INTERRUPT! bit {}, addr 0x{:04X}, IE: {:02X}, halted: {}",
        bit,
        addr,
        mmu.direct_read(IE_REG),
        mmu.reg.halted
    );
    */

    mmu.clear_if_reg_bits(bit);
    let pc = mmu.reg.pc;
    push_op(mmu, pc);
    mmu.reg.pc = addr;
    mmu.reg.ime = 0;
}

// Handles interrupt by checking for interrupt requests in correct order
// and trigger the interrupt handler after proper delay.
// Returns the triggered interrupt bit, or zero if no interrupt triggered.
pub fn handle_interrupts(mmu: &mut MMU) -> u8 {
    let if_reg = mmu.direct_read(IF_REG);
    let ie_reg = mmu.direct_read(IE_REG);
    let masked = if_reg & ie_reg;

    if masked != 0 {
        // If the CPU is halted it should always wake up when
        // an IF flag and the corresponding IE flag are both set.
        // This is true no matter the state of IME.
        mmu.wakeup_if_halted();
    }

    if mmu.reg.ime == 1 {
        mmu.reg.ime = 2;
        return 0;
    }

    if mmu.reg.ime == 2 {
        if masked & IF_VBLANK_BIT != 0 {
            interrupt(mmu, IF_VBLANK_BIT, VBLANK_ADDR);
            return IF_VBLANK_BIT;
        } else if masked & IF_LCDC_BIT != 0 {
            interrupt(mmu, IF_LCDC_BIT, LCDC_ADDR);
            return IF_LCDC_BIT;
        } else if masked & IF_TMR_BIT != 0 {
            interrupt(mmu, IF_TMR_BIT, TMR_ADDR);
            return IF_TMR_BIT;
        } else if masked & IF_SERIAL_BIT != 0 {
            interrupt(mmu, IF_SERIAL_BIT, SERIAL_ADDR);
            return IF_SERIAL_BIT;
        } else if masked & IF_INP_BIT != 0 {
            interrupt(mmu, IF_INP_BIT, INP_ADDR);
            return IF_INP_BIT;
        }
    }

    return 0;
}
