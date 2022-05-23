# RustBoy

A personal project to learn Rust and basic emulator development.

## PPU

The PPU is implemented on a scanline basis. There was an attempt
at a more accurate FIFO implementation, but as the documentation
of this diverges quite a bit and gives very little benefit compared
to a scanline-based approach I decided to not continue down that
path for now. This also has the benefit of being a lot faster.

Because of this a few demos and one game ("Prehistoric Man") will
not work as intended. If the knowledge and documentation about the
inner workings of the Gameboy improves in the future, I might take
another stab at the FIFO based approach.

## Blargg Test Suite

|       |       |       |       |       |       |       |       |       |       |       |       |       |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| cgb_sound | [:green_heart:](x "01-registers: PASS") | [:green_heart:](x "02-len ctr: PASS") | [:green_heart:](x "03-trigger: PASS") | [:green_heart:](x "04-sweep: PASS") | [:green_heart:](x "05-sweep details: PASS") | [:green_heart:](x "06-overflow on trigger: PASS") | [:green_heart:](x "07-len sweep period sync: PASS") | [:green_heart:](x "08-len ctr during power: PASS") | [:green_heart:](x "09-wave read while on: PASS") | [:green_heart:](x "10-wave trigger while on: PASS") | [:green_heart:](x "11-regs after power: PASS") | [:green_heart:](x "12-wave: PASS") |
| cpu_instrs | [:green_heart:](x "01-special: PASS") | [:green_heart:](x "02-interrupts: PASS") | [:green_heart:](x "03-op sp,hl: PASS") | [:green_heart:](x "04-op r,imm: PASS") | [:green_heart:](x "05-op rp: PASS") | [:green_heart:](x "06-ld r,r: PASS") | [:green_heart:](x "07-jr,jp,call,ret,rst: PASS") | [:green_heart:](x "08-misc instrs: PASS") | [:green_heart:](x "09-op r,r: PASS") | [:green_heart:](x "10-bit ops: PASS") | [:green_heart:](x "11-op a,(hl): PASS") |
| dmg_sound | [:green_heart:](x "01-registers: PASS") | [:green_heart:](x "02-len ctr: PASS") | [:green_heart:](x "03-trigger: PASS") | [:green_heart:](x "04-sweep: PASS") | [:green_heart:](x "05-sweep details: PASS") | [:green_heart:](x "06-overflow on trigger: PASS") | [:green_heart:](x "07-len sweep period sync: PASS") | [:green_heart:](x "08-len ctr during power: PASS") | [:green_heart:](x "09-wave read while on: PASS") | [:green_heart:](x "10-wave trigger while on: PASS") | [:green_heart:](x "11-regs after power: PASS") | [:green_heart:](x "12-wave write while on: PASS") |
| instr_timing | [:green_heart:](x "instr_timing: PASS") |
| interrupt_time | [🙅](x "interrupt_time: SKIPPED") |
| mem_timing | [:green_heart:](x "01-read_timing: PASS") | [:green_heart:](x "02-write_timing: PASS") | [:green_heart:](x "03-modify_timing: PASS") |
| mem_timing-2 | [:green_heart:](x "01-read_timing: PASS") | [:green_heart:](x "02-write_timing: PASS") | [:green_heart:](x "03-modify_timing: PASS") |
| oam_bug | [:red_circle:](x "1-lcd_sync: FAIL") | [:red_circle:](x "2-causes: FAIL") | [:green_heart:](x "3-non_causes: PASS") | [:red_circle:](x "4-scanline_timing: FAIL") | [:red_circle:](x "5-timing_bug: FAIL") | [:green_heart:](x "6-timing_no_bug: PASS") | [:red_circle:](x "7-timing_effect: FAIL") | [:red_circle:](x "8-instr_effect: FAIL") |


## Mooneye Test Suite

|       |       |       |       |       |       |       |       |       |       |       |       |       |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| acceptance | [:red_circle:](x "add_sp_e_timing: FAIL") | [:red_circle:](x "boot_div-S: FAIL") | [:red_circle:](x "boot_div-dmg0: FAIL") | [:red_circle:](x "boot_div-dmgABCmgb: FAIL") | [:red_circle:](x "boot_div2-S: FAIL") | [:red_circle:](x "boot_hwio-S: FAIL") | [:red_circle:](x "boot_hwio-dmg0: FAIL") | [:red_circle:](x "boot_hwio-dmgABCmgb: FAIL") | [:red_circle:](x "boot_regs-dmg0: FAIL") | [:green_heart:](x "boot_regs-dmgABC: PASS") | [:red_circle:](x "boot_regs-mgb: FAIL") | [:red_circle:](x "boot_regs-sgb: FAIL") |
| | [:red_circle:](x "boot_regs-sgb2: FAIL") | [:red_circle:](x "call_cc_timing: FAIL") | [:red_circle:](x "call_cc_timing2: FAIL") | [:red_circle:](x "call_timing: FAIL") | [:red_circle:](x "call_timing2: FAIL") | [:red_circle:](x "di_timing-GS: FAIL") | [:green_heart:](x "div_timing: PASS") | [:red_circle:](x "ei_sequence: FAIL") | [:red_circle:](x "ei_timing: FAIL") | [:green_heart:](x "halt_ime0_ei: PASS") | [:red_circle:](x "halt_ime0_nointr_timing: FAIL") | [:green_heart:](x "halt_ime1_timing: PASS") |
| | [:red_circle:](x "halt_ime1_timing2-GS: FAIL") | [:red_circle:](x "if_ie_registers: FAIL") | [:red_circle:](x "intr_timing: FAIL") | [:red_circle:](x "jp_cc_timing: FAIL") | [:red_circle:](x "jp_timing: FAIL") | [:red_circle:](x "ld_hl_sp_e_timing: FAIL") | [:red_circle:](x "oam_dma_restart: FAIL") | [🙅](x "oam_dma_start: SKIPPED") | [:red_circle:](x "oam_dma_timing: FAIL") | [:green_heart:](x "pop_timing: PASS") | [:red_circle:](x "push_timing: FAIL") | [:red_circle:](x "rapid_di_ei: FAIL") |
| | [:red_circle:](x "ret_cc_timing: FAIL") | [:red_circle:](x "ret_timing: FAIL") | [:red_circle:](x "reti_intr_timing: FAIL") | [:red_circle:](x "reti_timing: FAIL") | [:red_circle:](x "rst_timing: FAIL") |
| acceptance/bits | [:green_heart:](x "mem_oam: PASS") | [:green_heart:](x "reg_f: PASS") | [:red_circle:](x "unused_hwio-GS: FAIL") |
| acceptance/instr | [:green_heart:](x "daa: PASS") |
| acceptance/interrupts | [:red_circle:](x "ie_push: FAIL") |
| acceptance/oam_dma | [:green_heart:](x "basic: PASS") | [:green_heart:](x "reg_read: PASS") | [:green_heart:](x "sources-GS: PASS") |
| acceptance/ppu | [🙅](x "hblank_ly_scx_timing-GS: SKIPPED") | [🙅](x "intr_1_2_timing-GS: SKIPPED") | [🙅](x "intr_2_0_timing: SKIPPED") | [🙅](x "intr_2_mode0_timing: SKIPPED") | [🙅](x "intr_2_mode0_timing_sprites: SKIPPED") | [🙅](x "intr_2_mode3_timing: SKIPPED") | [🙅](x "intr_2_oam_ok_timing: SKIPPED") | [:red_circle:](x "lcdon_timing-GS: FAIL") | [:red_circle:](x "lcdon_write_timing-GS: FAIL") | [:red_circle:](x "stat_irq_blocking: FAIL") | [:red_circle:](x "stat_lyc_onoff: FAIL") | [🙅](x "vblank_stat_intr-GS: SKIPPED") |
| acceptance/serial | [:red_circle:](x "boot_sclk_align-dmgABCmgb: FAIL") |
| acceptance/timer | [:green_heart:](x "div_write: PASS") | [:red_circle:](x "rapid_toggle: FAIL") | [:green_heart:](x "tim00: PASS") | [:green_heart:](x "tim00_div_trigger: PASS") | [:green_heart:](x "tim01: PASS") | [:green_heart:](x "tim01_div_trigger: PASS") | [:green_heart:](x "tim10: PASS") | [:green_heart:](x "tim10_div_trigger: PASS") | [:green_heart:](x "tim11: PASS") | [:green_heart:](x "tim11_div_trigger: PASS") | [:red_circle:](x "tima_reload: FAIL") | [:red_circle:](x "tima_write_reloading: FAIL") |
| | [:red_circle:](x "tma_write_reloading: FAIL") |
| emulator-only/mbc1 | [:green_heart:](x "bits_bank1: PASS") | [:green_heart:](x "bits_bank2: PASS") | [:green_heart:](x "bits_mode: PASS") | [:green_heart:](x "bits_ramg: PASS") | [:green_heart:](x "multicart_rom_8Mb: PASS") | [:green_heart:](x "ram_256kb: PASS") | [:green_heart:](x "ram_64kb: PASS") | [:green_heart:](x "rom_16Mb: PASS") | [:green_heart:](x "rom_1Mb: PASS") | [:green_heart:](x "rom_2Mb: PASS") | [:green_heart:](x "rom_4Mb: PASS") | [:green_heart:](x "rom_512kb: PASS") |
| | [:green_heart:](x "rom_8Mb: PASS") |
| emulator-only/mbc2 | [:green_heart:](x "bits_ramg: PASS") | [:green_heart:](x "bits_romb: PASS") | [:green_heart:](x "bits_unused: PASS") | [:green_heart:](x "ram: PASS") | [:green_heart:](x "rom_1Mb: PASS") | [:green_heart:](x "rom_2Mb: PASS") | [:green_heart:](x "rom_512kb: PASS") |
| emulator-only/mbc5 | [:green_heart:](x "rom_16Mb: PASS") | [:green_heart:](x "rom_1Mb: PASS") | [:green_heart:](x "rom_2Mb: PASS") | [:green_heart:](x "rom_32Mb: PASS") | [:green_heart:](x "rom_4Mb: PASS") | [:green_heart:](x "rom_512kb: PASS") | [:green_heart:](x "rom_64Mb: PASS") | [:green_heart:](x "rom_8Mb: PASS") |


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
