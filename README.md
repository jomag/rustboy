RustBoy
=======

A personal project to learn Rust and basic emulator development.

Blargg tests
------------

Slowly this emulator learns to pass more and more Blargg tests.

Currently the following tests has been tried:

### CPU instructions

 * `01-special.gb`: fails
 * `02-interrupts.gb`: fails
 * `03-op sp,hl.gb`: fails
 * `04-op r,imm.gb`: fails
 * `05-op rp.gb`: **passes!**
 * `06-ld r,r.gb`: **passes!**
 * `07-jr,jp,call,ret,rst.gb`: **passes!**
 * `08-misc instrs.gb`: **passes!**
 * `09-op r,r.gb`: fails
 * `10-bit ops.gb`: **passes!**
 * `11-op a,(hl).gb`: fails

Graphics
--------

Resolution: 160 x 144
Real resolution: 256 x 256
Tiles: 32 x 32
Real resolution: 256 x 256 (32 * 8 x 32 * 8)
Clock speed: 4.194304 MHz (2 ** 22)
Vertical sync: 59.73 Hz

References
----------

Pandocs ("Everything You Always Wanted To Know About GAMEBOY")
http://bgb.bircd.org/pandocs.htm

Z80 instruction set:
http://clrhome.org/table/

Disassembly of boot ROM: https://gist.github.com/drhelius/6063288

Blog: Why did I spend 1.5 months creating a Gameboy emulator?
https://blog.rekawek.eu/2017/02/09/coffee-gb/

The Ultimate Gameboy Talk
https://www.youtube.com/watch?v=HyzD8pNlpwI&t=29m12s

Emulating the Gameboy:
http://www.codeslinger.co.uk/pages/projects/gameboy.html
