# RustBoy

A personal project to learn Rust and basic emulator development.

## Blargg tests

Slowly this emulator learns to pass more and more Blargg tests.

Currently the following tests has been tried:

### CPU instructions

* `01-special.gb`: **passes!**
* `02-interrupts.gb`: fails
* `03-op sp,hl.gb`: **passes!**
* `04-op r,imm.gb`: **passes!**
* `05-op rp.gb`: **passes!**
* `06-ld r,r.gb`: **passes!**
* `07-jr,jp,call,ret,rst.gb`: **passes!**
* `08-misc instrs.gb`: **passes!**
* `09-op r,r.gb`: **passes!**
* `10-bit ops.gb`: **passes!**
* `11-op a,(hl).gb`: **passes!**

## Mooneye GB

The Mooneye GB emulator (which also happens to be written in Rust)
includes a number of tests as well. Here's current state of some:

* `acceptance/bits/mem_oam`: **passes!**
* `acceptance/bits/reg_f`: **passes!**
* `acceptance/bits/unused_hwio-GS`: fails

* `acceptance/instr/daa`: **passes!**

* `acceptance/interrupts/ie_push`: fails with "R1: not cancelled"

* `acceptance/oam_dma/basic`: **passes!**
* `acceptance/oam_dma/reg_read`: **passes!**
* `acceptance/oam_dma/sources-dmgABCmgbS`: fails

* `acceptance/timer/div_write`: **passes!**
* `acceptance/timer/rapid_toggle`: fails
* `acceptance/timer/tim00`: **passes!**
* `acceptance/timer/tim00_div_trigger`: **passes!**
* `acceptance/timer/tim01`: **passes!**
* `acceptance/timer/tim01_div_trigger`: **passes!**
* `acceptance/timer/tim10`: **passes!**
* `acceptance/timer/tim10_div_trigger`: **passes!**
* `acceptance/timer/tim11`: **passes!**
* `acceptance/timer/tim11_div_trigger`: **passes!**
* `acceptance/timer/tima_reload`: fails
* `acceptance/timer/tima_write_reloading`: fails
* `acceptance/timer/tma_write_reloading`: fails

* `acceptance/add_sp_e_timing`: **passes!**
* `acceptance/boot_regs-dmgABC.gb`: **passes!**
* `acceptance/call_cc_timing`: **passes!**
* `acceptance/call_cc_timing2`: **passes!**
* `acceptance/call_timing`: **passes!**
* `acceptance/call_timing2`: **passes!**
* `acceptance/div_timing`: **passes!**
* `acceptance/ei_sequence`: fails (used to pass?)
* `acceptance/ei_timing`: fails (used to pass?)
* `acceptance/halt_ime0_ei`: fails
* `acceptance/halt_ime0_nointr_timing`: fails
* `acceptance/halt_ime1_timing`: fails
* `acceptance/if_ei_registers`: fails (because serial interrupt not impl.)
* `acceptance/intr_timing`: fails: round 1
* `acceptance/jp_cc_timing`: **passes!**
* `acceptance/jp_timing`: **passes!**
* `acceptance/ld_hl_sp_e_timing`: **passes!**
* `acceptance/oam_dma_restart`: **passes!**
* `acceptance/oam_dma_start`: **passes!**
* `acceptance/oam_dma_timing`: **passes!**
* `acceptance/pop_timing`: **passes!**
* `acceptance/push_timing`: **passes!**
* `acceptance/rapid_di_ei`: fails! (used to pass?)
* `acceptance/ret_cc_timing`: **passes!**
* `acceptance/reti_intr_timing`: fails! (used to pass?)
* `acceptance/reti_timing`: **passes!**
* `acceptance/ret_timing`: **passes!**
* `acceptance/rst_timing`: **passes!**

## Graphics

Resolution: 160 x 144
Real resolution: 256 x 256
Tiles: 32 x 32
Real resolution: 256 x 256 (32 x (8 x 32) x 8)
Clock speed: 4.194304 MHz (2 ** 22)
Vertical sync: 59.73 Hz

## References

Pandocs ("Everything You Always Wanted To Know About GAMEBOY")
<http://bgb.bircd.org/pandocs.htm>

Z80 instruction set:
<http://clrhome.org/table/>

Disassembly of boot ROM:
<https://gist.github.com/drhelius/6063288>

Blog: Why did I spend 1.5 months creating a Gameboy emulator?
<https://blog.rekawek.eu/2017/02/09/coffee-gb/>

The Ultimate Gameboy Talk
<https://www.youtube.com/watch?v=HyzD8pNlpwI&t=29m12s>

Emulating the Gameboy:
<http://www.codeslinger.co.uk/pages/projects/gameboy.html>

The Cycle-Accurate Game Boy Docs:
<https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf>
