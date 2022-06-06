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

{% include "./blargg.md" %}

## Mooneye Test Suite

{% include "./mooneye.md" %}

## DMG ACID2

{% include "./dmg-acid2.md" %}

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
