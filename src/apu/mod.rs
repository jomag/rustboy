// APU resources:
//
// Pan Doc:
// http://bgb.bircd.org/pandocs.htm#soundoverview
//
// Game Boy Sound Operation by Blargg:
// https://gist.github.com/drhelius/3652407
//
// GB Sound Emulation by Nightshade:
// https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
//

// TODO:
// - For DMG hardware only, the length counters should be usable even
//   when NR52 is powered off. See the Blargg doc above.
// - After the sound hardware is powered on, frame sequencer should be
//   reset so next step is step 0.
// - Remove duplicated envelope code

pub mod apu;
pub mod dac;
pub mod length_counter;
pub mod noise_gen;
pub mod square_gen;
pub mod sweep;
pub mod wave_gen;
