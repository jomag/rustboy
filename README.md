# RustBoy

A personal project to learn Rust and basic emulator development.

## Blargg tests

Slowly this emulator learns to pass more and more Blargg tests.

Currently the following tests has been tried:

### CPU instructions

- `instr_timing.gb`: **pass**
- `interrupt_time.gb`: fail
- `mem_timing/mem_timing.gb`: **pass**
- `mem_timing-2/mem_timing.gb`: **pass**

Individual memory timing tests:

- `mem_timing/01-read_timing.gb`: **pass**
- `mem_timing/02-write_timing.gb`: **pass**
- `mem_timing/03-modify_timing.gb`: **pass**

Individual CPU instruction tests:

- `01-special.gb`: **pass**
- `02-interrupts.gb`: **pass**
- `03-op sp,hl.gb`: **pass**
- `04-op r,imm.gb`: **pass**
- `05-op rp.gb`: **pass**
- `06-ld r,r.gb`: **pass**
- `07-jr,jp,call,ret,rst.gb`: **pass**
- `08-misc instrs.gb`: **pass**
- `09-op r,r.gb`: **pass**
- `10-bit ops.gb`: **pass**
- `11-op a,(hl).gb`: **pass**

## Mooneye GB

The Mooneye GB emulator (which also happens to be written in Rust)
includes a number of tests as well. Here's current state of some:

- `acceptance/bits/mem_oam`: **pass**
- `acceptance/bits/reg_f`: **pass**
- `acceptance/bits/unused_hwio-GS`: fail

- `acceptance/instr/daa`: **pass**

- `acceptance/interrupts/ie_push`: fail with "R1: not cancelled"

- `acceptance/oam_dma/basic`: **pass**
- `acceptance/oam_dma/reg_read`: **pass**
- `acceptance/oam_dma/sources-dmgABCmgbS`: **pass**

- `acceptance/timer/div_write`: **pass**
- `acceptance/timer/rapid_toggle`: fail
- `acceptance/timer/tim00`: **pass**
- `acceptance/timer/tim00_div_trigger`: **pass**
- `acceptance/timer/tim01`: **pass**
- `acceptance/timer/tim01_div_trigger`: **pass**
- `acceptance/timer/tim10`: **pass**
- `acceptance/timer/tim10_div_trigger`: **pass**
- `acceptance/timer/tim11`: **pass**
- `acceptance/timer/tim11_div_trigger`: **pass**
- `acceptance/timer/tima_reload`: fail
- `acceptance/timer/tima_write_reloading`: fail
- `acceptance/timer/tma_write_reloading`: fail

- `acceptance/add_sp_e_timing`: **pass**
- `acceptance/boot_regs-dmgABC.gb`: **pass**
- `acceptance/call_cc_timing`: **pass**
- `acceptance/call_cc_timing2`: **pass**
- `acceptance/call_timing`: **pass**
- `acceptance/call_timing2`: **pass**
- `acceptance/div_timing`: **pass**
- `acceptance/ei_sequence`: fail (used to pass?)
- `acceptance/ei_timing`: fail (used to pass?)
- `acceptance/halt_ime0_ei`: fail
- `acceptance/halt_ime0_nointr_timing`: fail
- `acceptance/halt_ime1_timing`: fail
- `acceptance/if_ei_registers`: fail (because serial interrupt not impl.)
- `acceptance/intr_timing`: fail: round 1
- `acceptance/jp_cc_timing`: **pass**
- `acceptance/jp_timing`: **pass**
- `acceptance/ld_hl_sp_e_timing`: **pass**
- `acceptance/oam_dma_restart`: **pass**
- `acceptance/oam_dma_start`: **pass**
- `acceptance/oam_dma_timing`: **pass**
- `acceptance/pop_timing`: **pass**
- `acceptance/push_timing`: **pass**
- `acceptance/rapid_di_ei`: fail! (used to pass?)
- `acceptance/ret_cc_timing`: **pass**
- `acceptance/reti_intr_timing`: fail! (used to pass?)
- `acceptance/reti_timing`: **pass**
- `acceptance/ret_timing`: **pass**
- `acceptance/rst_timing`: **pass**

## Graphics

Resolution: 160 x 144
Real resolution: 256 x 256
Tiles: 32 x 32
Real resolution: 256 x 256 (32 x (8 x 32) x 8)
Clock speed: 4.194304 MHz (2 \*\* 22)
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
