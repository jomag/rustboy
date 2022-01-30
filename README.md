# RustBoy

A personal project to learn Rust and basic emulator development.

## Blargg Test Suite

|       |       |       |       |       |       |       |       |       |       |       |       |       |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| cgb_sound | [:green_heart:](x "01-registers: PASS") | [:red_circle:](x "02-len ctr: FAIL") | [:red_circle:](x "03-trigger: FAIL") | [🙅](x "04-sweep: SKIPPED") | [🙅](x "05-sweep details: SKIPPED") | [:red_circle:](x "06-overflow on trigger: FAIL") | [🙅](x "07-len sweep period sync: SKIPPED") | [:red_circle:](x "08-len ctr during power: FAIL") | [:red_circle:](x "09-wave read while on: FAIL") | [:green_heart:](x "10-wave trigger while on: PASS") | [🙅](x "11-regs after power: SKIPPED") | [:red_circle:](x "12-wave: FAIL") |
| cpu_instrs | [:green_heart:](x "01-special: PASS") | [:green_heart:](x "02-interrupts: PASS") | [:green_heart:](x "03-op sp,hl: PASS") | [:green_heart:](x "04-op r,imm: PASS") | [:green_heart:](x "05-op rp: PASS") | [:green_heart:](x "06-ld r,r: PASS") | [:green_heart:](x "07-jr,jp,call,ret,rst: PASS") | [:green_heart:](x "08-misc instrs: PASS") | [:green_heart:](x "09-op r,r: PASS") | [:green_heart:](x "10-bit ops: PASS") | [:green_heart:](x "11-op a,(hl): PASS") |
| dmg_sound | [:green_heart:](x "01-registers: PASS") | [:red_circle:](x "02-len ctr: FAIL") | [:red_circle:](x "03-trigger: FAIL") | [🙅](x "04-sweep: SKIPPED") | [🙅](x "05-sweep details: SKIPPED") | [:red_circle:](x "06-overflow on trigger: FAIL") | [🙅](x "07-len sweep period sync: SKIPPED") | [:red_circle:](x "08-len ctr during power: FAIL") | [:red_circle:](x "09-wave read while on: FAIL") | [:red_circle:](x "10-wave trigger while on: FAIL") | [🙅](x "11-regs after power: SKIPPED") | [:red_circle:](x "12-wave write while on: FAIL") |
| instr_timing | [:green_heart:](x "instr_timing: PASS") |
| interrupt_time | [:red_circle:](x "interrupt_time: FAIL") |
| mem_timing | [:green_heart:](x "01-read_timing: PASS") | [:green_heart:](x "02-write_timing: PASS") | [:green_heart:](x "03-modify_timing: PASS") |
| mem_timing-2 | [:green_heart:](x "01-read_timing: PASS") | [:green_heart:](x "02-write_timing: PASS") | [:green_heart:](x "03-modify_timing: PASS") |
| oam_bug | [:red_circle:](x "1-lcd_sync: FAIL") | [:red_circle:](x "2-causes: FAIL") | [:green_heart:](x "3-non_causes: PASS") | [:red_circle:](x "4-scanline_timing: FAIL") | [:red_circle:](x "5-timing_bug: FAIL") | [:green_heart:](x "6-timing_no_bug: PASS") | [:red_circle:](x "7-timing_effect: FAIL") | [:red_circle:](x "8-instr_effect: FAIL") |


## Mooneye Test Suite

|       |       |       |       |       |       |       |       |       |       |       |       |       |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| acceptance | [:green_heart:](x "add_sp_e_timing: PASS") | [:red_circle:](x "boot_div-S: FAIL") | [:red_circle:](x "boot_div-dmg0: FAIL") | [:red_circle:](x "boot_div-dmgABCmgb: FAIL") | [:red_circle:](x "boot_div2-S: FAIL") | [:red_circle:](x "boot_hwio-S: FAIL") | [:red_circle:](x "boot_hwio-dmg0: FAIL") | [:red_circle:](x "boot_hwio-dmgABCmgb: FAIL") | [:red_circle:](x "boot_regs-dmg0: FAIL") | [:green_heart:](x "boot_regs-dmgABC: PASS") | [:red_circle:](x "boot_regs-mgb: FAIL") | [:red_circle:](x "boot_regs-sgb: FAIL") |
| | [:red_circle:](x "boot_regs-sgb2: FAIL") | [:green_heart:](x "call_cc_timing: PASS") | [:green_heart:](x "call_cc_timing2: PASS") | [:green_heart:](x "call_timing: PASS") | [:green_heart:](x "call_timing2: PASS") | [:red_circle:](x "di_timing-GS: FAIL") | [:green_heart:](x "div_timing: PASS") | [:red_circle:](x "ei_sequence: FAIL") | [:red_circle:](x "ei_timing: FAIL") | [:green_heart:](x "halt_ime0_ei: PASS") | [:red_circle:](x "halt_ime0_nointr_timing: FAIL") | [:green_heart:](x "halt_ime1_timing: PASS") |
| | [:red_circle:](x "halt_ime1_timing2-GS: FAIL") | [:red_circle:](x "if_ie_registers: FAIL") | [:red_circle:](x "intr_timing: FAIL") | [:green_heart:](x "jp_cc_timing: PASS") | [:green_heart:](x "jp_timing: PASS") | [:green_heart:](x "ld_hl_sp_e_timing: PASS") | [:green_heart:](x "oam_dma_restart: PASS") | [:green_heart:](x "oam_dma_start: PASS") | [:green_heart:](x "oam_dma_timing: PASS") | [:green_heart:](x "pop_timing: PASS") | [:green_heart:](x "push_timing: PASS") | [:red_circle:](x "rapid_di_ei: FAIL") |
| | [:green_heart:](x "ret_cc_timing: PASS") | [:green_heart:](x "ret_timing: PASS") | [:red_circle:](x "reti_intr_timing: FAIL") | [:green_heart:](x "reti_timing: PASS") | [:green_heart:](x "rst_timing: PASS") |
| acceptance/timer | [:green_heart:](x "div_write: PASS") | [:red_circle:](x "rapid_toggle: FAIL") | [:green_heart:](x "tim00: PASS") | [:green_heart:](x "tim00_div_trigger: PASS") | [:green_heart:](x "tim01: PASS") | [:green_heart:](x "tim01_div_trigger: PASS") | [:green_heart:](x "tim10: PASS") | [:green_heart:](x "tim10_div_trigger: PASS") | [:green_heart:](x "tim11: PASS") | [:green_heart:](x "tim11_div_trigger: PASS") | [:red_circle:](x "tima_reload: FAIL") | [:red_circle:](x "tima_write_reloading: FAIL") |
| | [:red_circle:](x "tma_write_reloading: FAIL") |
| acceptance/serial | [:red_circle:](x "boot_sclk_align-dmgABCmgb: FAIL") |
| acceptance/ppu | [:red_circle:](x "hblank_ly_scx_timing-GS: FAIL") | [🙅](x "intr_1_2_timing-GS: SKIPPED") | [:red_circle:](x "intr_2_0_timing: FAIL") | [:red_circle:](x "intr_2_mode0_timing: FAIL") | [:red_circle:](x "intr_2_mode0_timing_sprites: FAIL") | [:red_circle:](x "intr_2_mode3_timing: FAIL") | [:red_circle:](x "intr_2_oam_ok_timing: FAIL") | [:red_circle:](x "lcdon_timing-GS: FAIL") | [:red_circle:](x "lcdon_write_timing-GS: FAIL") | [:red_circle:](x "stat_irq_blocking: FAIL") | [:red_circle:](x "stat_lyc_onoff: FAIL") | [:red_circle:](x "vblank_stat_intr-GS: FAIL") |
| acceptance/interrupts | [:red_circle:](x "ie_push: FAIL") |
| acceptance/bits | [:green_heart:](x "mem_oam: PASS") | [:green_heart:](x "reg_f: PASS") | [:red_circle:](x "unused_hwio-GS: FAIL") |
| acceptance/instr | [:green_heart:](x "daa: PASS") |
| acceptance/oam_dma | [:green_heart:](x "basic: PASS") | [:green_heart:](x "reg_read: PASS") | [:red_circle:](x "sources-GS: FAIL") |


## Graphics

- Resolution: 160 x 144
- Real resolution: 256 x 256
- Tiles: 32 x 32
- Real resolution: 256 x 256 (32 x (8 x 32) x 8)
- Clock speed: 4.194304 MHz (2 \*\* 22)
- Vertical sync: 59.73 Hz

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
