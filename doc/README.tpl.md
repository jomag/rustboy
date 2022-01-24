# RustBoy

A personal project to learn Rust and basic emulator development.

## Blargg tests

Slowly this emulator learns to pass more and more Blargg tests.

{% include "./blargg.md" %}

## Mooneye GB

The Mooneye GB emulator (w`hich also happens to be written in Rust)
includes a number of tests as well. Here's current state of some:

{% include "./mooneye.md" %}

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
