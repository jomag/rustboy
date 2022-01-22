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

The Mooneye GB emulator (w`hich also happens to be written in Rust)
includes a number of tests as well. Here's current state of some:

## Mooneye Test Suite

### Acceptance tests

|       |       |       |       |       |       |       |       |       |       |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| [:green_heart:](x "rst_timing.gb") | [:green_heart:](x "jp_timing.gb") | [:green_heart:](x "reti_timing.gb") | [:green_heart:](x "oam_dma_timing.gb") | [:red_circle:](x "if_ie_registers.gb") | [:red_circle:](x "boot_regs-mgb.gb") | [:red_circle:](x "rapid_di_ei.gb") | [:green_heart:](x "oam_dma_restart.gb") | [:green_heart:](x "pop_timing.gb") | [:green_heart:](x "jp_cc_timing.gb") |
| [:green_heart:](x "oam_dma_start.gb") | [:green_heart:](x "push_timing.gb") | [:red_circle:](x "boot_regs-sgb2.gb") | [:green_heart:](x "add_sp_e_timing.gb") | [:red_circle:](x "di_timing-GS.gb") | [:red_circle:](x "intr_timing.gb") | [:green_heart:](x "ld_hl_sp_e_timing.gb") | [:red_circle:](x "boot_div2-S.gb") | [:red_circle:](x "boot_div-S.gb") | [:red_circle:](x "reti_intr_timing.gb") |
| [:green_heart:](x "call_cc_timing2.gb") | [:red_circle:](x "halt_ime0_nointr_timing.gb") | [:green_heart:](x "div_timing.gb") | [:green_heart:](x "halt_ime0_ei.gb") | [:red_circle:](x "boot_hwio-S.gb") | [:green_heart:](x "call_cc_timing.gb") | [:red_circle:](x "ei_sequence.gb") | [:green_heart:](x "boot_regs-dmgABC.gb") | [:green_heart:](x "ret_cc_timing.gb") | [:red_circle:](x "boot_div-dmg0.gb") |
| [:red_circle:](x "halt_ime1_timing2-GS.gb") | [:red_circle:](x "boot_div-dmgABCmgb.gb") | [:red_circle:](x "boot_hwio-dmgABCmgb.gb") | [:red_circle:](x "boot_regs-dmg0.gb") | [:green_heart:](x "call_timing2.gb") | [:green_heart:](x "call_timing.gb") | [:red_circle:](x "ei_timing.gb") | [:red_circle:](x "boot_hwio-dmg0.gb") | [:red_circle:](x "boot_regs-sgb.gb") | [:green_heart:](x "halt_ime1_timing.gb") |
| [:green_heart:](x "ret_timing.gb") |


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
