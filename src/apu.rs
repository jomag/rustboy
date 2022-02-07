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
// - Need to differentiate between a channel being "enabled" and the
//   DAC being enabled?
// - Remove duplicated envelope code

use ringbuf::Producer;

use crate::{
    emu::Machine,
    mmu::{
        NR10_REG, NR11_REG, NR12_REG, NR13_REG, NR14_REG, NR20_REG, NR21_REG, NR22_REG, NR23_REG,
        NR24_REG, NR30_REG, NR31_REG, NR32_REG, NR33_REG, NR34_REG, NR40_REG, NR41_REG, NR42_REG,
        NR43_REG, NR44_REG, NR50_REG, NR51_REG, NR52_REG,
    },
};

// All channels have a length counter which counts down and disables
// the channel when it reaches zero. The length counter can be
// disabled.
pub struct LengthCounter {
    // Max length value. 256 for ch 3 (wave), 64 for the others
    max: u16,

    machine: Machine,
    _enabled: bool,
    pub value: u16,
}

impl LengthCounter {
    fn new(machine: Machine, max: u16) -> Self {
        LengthCounter {
            machine,
            max,
            _enabled: false,
            value: 0,
        }
    }

    fn power_off(&mut self) {
        self._enabled = false;
        self.value = match self.machine {
            Machine::GameBoyDMG => self.value,
            Machine::GameBoyCGB => 0,
            _ => panic!("unsupported machine type"),
        }
    }

    pub fn write_reg_nrx1(&mut self, value: u8) {
        let mask = self.max - 1;
        self.value = self.max - (value as u16 & mask);
    }

    pub fn next_seq_step_will_not_count_down(seq_step: u8) -> bool {
        return seq_step % 2 == 0;
    }

    pub fn is_enabled(&self) -> bool {
        self._enabled
    }

    // Enable the counter. If the next sequencer step will not clock
    // the length counter, the counter value is immediately decremented
    // which may cause the channel to become disabled.
    //
    // If this function returns true, the channel should be disabled.
    pub fn enable(&mut self, en: bool, seq_step: u8) -> bool {
        if en {
            if !self._enabled {
                self._enabled = true;
                if LengthCounter::next_seq_step_will_not_count_down(seq_step) && self.value > 0 {
                    return self.count_down();
                }
            }
        } else {
            self._enabled = false;
        }

        false
    }

    // When triggered, if length counter is 0 it should
    // be reset to 64 (256 for wave channel).
    //
    // Obscure behavior:
    // If channel is triggered when next sequencer step will not
    // clock the length counter, the length counter is immediately
    // decremented.
    fn trigger(&mut self, reset_value: u16, seq_step: u8) {
        if self.value == 0 {
            self.value = reset_value;

            if self._enabled && LengthCounter::next_seq_step_will_not_count_down(seq_step) {
                self.value -= 1;
            }
        }
    }

    // This function should be called by the 256 Hz frame sequencer
    // tick. If it returns true, it has reached zero and the channel
    // should be disabled.
    fn count_down(&mut self) -> bool {
        if self._enabled {
            match self.value {
                0 => return false,
                1 => {
                    self.value = 0;
                    return true;
                }
                _ => {
                    self.value -= 1;
                    return false;
                }
            }
        } else {
            return false;
        }
    }
}

// Every channel has a DAC: a 4-bit digital-to-analog converter
// that generates a voltage from -1 to +1 for values 0 to 15.
pub struct DAC {
    powered_on: bool,
}

impl DAC {
    fn new() -> Self {
        DAC { powered_on: false }
    }

    fn convert(&self, inp: u8) -> f32 {
        assert!(inp & 0xF0 == 0);
        match self.powered_on {
            true => inp as f32 * (2.0 / 15.0) - 1.0,
            false => 0.0,
        }
    }
}

pub struct Sweep {
    duration: u8,
    decrement: bool,
    shift: u8,
    enabled: bool,
    counter: u8,
    shadow_frequency: u16,

    // To handle obscure behavior. See write_reg_nr10.
    has_calculated_in_decrement_mode: bool,
}

impl Sweep {
    fn new() -> Self {
        Sweep {
            // NR10, bit 6..4
            duration: 0,

            // NR10, bit 3
            decrement: false,

            // NR10, bit 3..0
            shift: 0,

            enabled: false,
            shadow_frequency: 0,
            counter: 0,

            has_calculated_in_decrement_mode: false,
        }
    }

    fn power_off(&mut self) {
        self.duration = 0;
        self.decrement = false;
        self.shift = 0;
        self.counter = 0;
    }

    fn read_reg_nr10(&self) -> u8 {
        let v = (self.duration << 4) | self.shift;
        if self.decrement {
            v | 0b1000
        } else {
            v
        }
    }

    fn write_reg_nr10(&mut self, value: u8, channel_enabled: &mut bool) {
        self.duration = (value >> 4) & 0b111;
        self.shift = value & 0b111;

        let prev = self.decrement;
        self.decrement = (value & 0b1000) != 0;
        if prev && !self.decrement && self.has_calculated_in_decrement_mode {
            // Obscure behavior: if the decrement bit is cleared after at least
            // one frequency calculation has been made in decrement mode
            // since the last trigger, the channel is immediately disabled.
            *channel_enabled = false;
        }
    }

    fn load_counter(&mut self) {
        self.counter = match self.duration {
            0 => 8,
            n => n,
        }
    }

    fn trigger(&mut self, channel_enabled: &mut bool, frequency: &mut u16) {
        self.shadow_frequency = *frequency;
        self.load_counter();
        self.enabled = self.duration != 0 || self.shift != 0;
        self.has_calculated_in_decrement_mode = false;

        if self.shift != 0 {
            // Overflow check
            self.calc_frequency(channel_enabled);
        }
    }

    fn calc_frequency(&mut self, channel_enabled: &mut bool) -> u16 {
        let mut f = self.shadow_frequency >> self.shift;

        if self.decrement {
            f = self.shadow_frequency - f;
            self.has_calculated_in_decrement_mode = true;
        } else {
            f = self.shadow_frequency + f;
        }

        if f > 2047 {
            *channel_enabled = false;
        }

        return f;
    }

    fn tick_128hz(&mut self, channel_enabled: &mut bool, frequency: &mut u16) {
        if self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                self.counter = self.duration;
                if self.counter == 0 {
                    self.counter = 8;
                }

                if self.enabled && self.duration > 0 {
                    let new_frequency = self.calc_frequency(channel_enabled);

                    if new_frequency < 2048 && self.shift > 0 {
                        *frequency = new_frequency;
                        self.shadow_frequency = new_frequency;

                        self.calc_frequency(channel_enabled);
                    }
                }
            }
        }
    }
}

// SquareWaveSoundGenerator
// ------------------------
//
// Note that this type is used for both sound channel 1 and 2.
// The only difference is that channel 2 does not have any sweep
// generator and the registers starts at NR20 instead of NR10.
// To differentiate between them, there's the `with_sweep` boolean.
//
// ---------
// Registers
// ---------
//
// NR10 (0xFF10): Sweep. Only for sound channel 1.
// - bit 6..4: sweep time
// - bit 3:    sweep direction
// - bit 2..0: number of sweep shifts
//
// NR11 (0xFF11), NR21 (0xFF16): Wave pattern and sound length
// - bit 7..6: wave pattern, aka "duty".
// - bit 5..0: sound length (write only)
//
// NR12 (0xFF12), NR22 (0xFF17): Envelope
// - bit 7..4: initial volume
// - bit 3:    envelope direction
// - bit 2..0: number of envelope sweeps
//
// NR13 (0xFF13), NR23 (0xFF18): lo bits of frequency
// - bit 7..0: lo bits of frequency (write only)
//
// NR14 (0xFF14), NR24 (0xFF19): hi bits of frequency + more
// - bit 7: initial, 1 = restart sound (write only)
// - bit 6: length counter/consecutive selection
// - bit 2..0: hi bits of frequency (write only)
//
pub struct SquareWaveSoundGenerator {
    // Frequency. 10 bits. Bit 7..0 in NR13 + bit 9..8 in NR14.
    frequency: u16,

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value (NR13, NR14) and
    // wave_duty_position moves to next position
    frequency_timer: u16,

    // Duty Cycle Pattern. Bit 7..6 of NR11. R/W.
    duty: usize,

    // Internal register. Holds a value between 0 and 7
    // and decides if the current output of the square wave
    // should be high or low. One of the following patterns
    // are selected with NR11 and the output is high if
    // the corresponding bit is high:
    //
    // 0: 12.5% - 00000001
    // 1: 25%   - 00000011
    // 2: 50%   - 00001111
    // 3: 75%   - 00111111
    //
    wave_duty_position: u16,

    // Initial volume of the envelope filter. Bit 7..4 in NR12
    initial_volume: u8,

    // Direction of the envelope. Bit 3 in NR12.
    // 0 (false) = decreasing, 1 (true) = increasing.
    envelope_increasing: bool,

    // Period count to initate envelope filter with on trigger.
    // Bits 2..0 of NR12.
    // FIXME: should there *be* a separate start-value,
    // or does the value count down and must be reset
    // before each trigger?
    envelope_periods_initial: u8,

    // Current volume of the envelope filter. Internal register.
    pub envelope: u8,

    // The envelope change period counter. Internal register.
    envelope_period: u8,

    // Internal enabled flag.
    pub enabled: bool,

    // Sweep is only enabled for the first square wave channel
    sweep: Option<Sweep>,

    pub length_counter: LengthCounter,
    pub dac: DAC,
    machine: Machine,
}

const WAVE_DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

impl SquareWaveSoundGenerator {
    pub fn new(with_sweep: bool, machine: Machine) -> Self {
        SquareWaveSoundGenerator {
            machine,
            enabled: false,
            envelope: 0,
            envelope_period: 0,
            envelope_periods_initial: 0,
            envelope_increasing: false,
            initial_volume: 0,
            length_counter: LengthCounter::new(machine, 64),
            frequency: 0,
            frequency_timer: 0,
            duty: 0,
            wave_duty_position: 0,
            sweep: if with_sweep { Some(Sweep::new()) } else { None },
            dac: DAC::new(),
        }
    }

    pub fn power_off(&mut self) {
        self.enabled = false;
        self.envelope = 0;
        self.envelope_period = 0;
        self.envelope_periods_initial = 0;
        self.envelope_increasing = false;
        self.initial_volume = 0;
        self.length_counter.power_off();
        self.frequency = 0;
        self.frequency_timer = 0;
        self.duty = 0;
        self.wave_duty_position = 0;
        self.dac = DAC::new();

        if let Some(ref mut sweep) = self.sweep {
            sweep.power_off();
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            NR10_REG | NR20_REG => match self.sweep {
                Some(ref sweep) => sweep.read_reg_nr10() | 0x80,
                None => 0xFF,
            },
            NR11_REG | NR21_REG => ((self.duty as u8) << 6) | 0b0011_1111,
            NR12_REG | NR22_REG => {
                let v = (self.initial_volume << 4) | (self.envelope_periods_initial & 0b111);
                if self.envelope_increasing {
                    v | 0b1000
                } else {
                    v
                }
            }
            NR13_REG | NR23_REG => 0xFF,
            NR14_REG | NR24_REG => {
                if self.length_counter.is_enabled() {
                    0xFF
                } else {
                    0b1011_1111
                }
            }
            _ => panic!(
                "Invalid register in square wave sound generator: 0x{:04x}",
                address
            ),
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8, seq_step: u8, powered_on: bool) {
        // If unpowered, all writes should be ignored except
        // length value if the machine is original Gameboy DMG
        if !powered_on {
            if matches!(self.machine, Machine::GameBoyDMG) {
                if address == NR11_REG || address == NR21_REG {
                    self.length_counter.write_reg_nrx1(value);
                }
            }
            return;
        }

        match address {
            NR10_REG | NR20_REG => {
                if let Some(ref mut sweep) = self.sweep {
                    sweep.write_reg_nr10(value, &mut self.enabled);
                }
            }
            NR11_REG | NR21_REG => {
                self.duty = ((value >> 6) & 3) as usize;
                self.length_counter.write_reg_nrx1(value);
            }
            NR12_REG | NR22_REG => {
                self.initial_volume = (value >> 4) & 0xF;
                self.dac.powered_on = value & 0b1111_1000 != 0;
                self.enabled = self.enabled && self.dac.powered_on;
                self.envelope_increasing = (value & 0b1000) != 0;
                self.envelope_periods_initial = value & 0b111;
            }
            NR13_REG | NR23_REG => {
                self.frequency = (self.frequency & 0b11_0000_0000) | value as u16
            }
            NR14_REG | NR24_REG => {
                self.frequency =
                    (self.frequency & 0b00_1111_1111) | (((value & 0b111) as u16) << 8);

                if self
                    .length_counter
                    .enable(value & 0b0100_0000 != 0, seq_step)
                {
                    self.enabled = false;
                }

                if value & 0b1000_0000 != 0 {
                    self.trigger(seq_step);
                }
            }
            _ => panic!("Invalid APU register: 0x{:04x}", address),
        }
    }

    fn trigger(&mut self, seq_step: u8) {
        self.enabled = true;
        self.length_counter.trigger(64, seq_step);
        self.frequency_timer = (2048 - self.frequency) * 4;
        self.envelope_period = self.envelope_periods_initial;
        self.envelope = self.initial_volume;

        if let Some(ref mut sweep) = self.sweep {
            sweep.trigger(&mut self.enabled, &mut self.frequency);
        }

        // FIXME: add sweep trigger handling

        // If DAC is not powered, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update(&mut self, hz64: bool, hz128: bool, hz256: bool) -> f32 {
        // Decrement frequency timer
        // FIXME: Handle frequency timer being less than 4.
        //        When so, add to the frequency timer instead:
        //        `if (tmr <= 4) { tmr += freq } else { tmr -= 4 }`
        if self.frequency_timer >= 4 {
            self.frequency_timer -= 4;

            // If frequency timer reaches 0, reset it to the selected frequency
            // (NR13, NR14) and increment the wave duty position
            if self.frequency_timer == 0 {
                self.frequency_timer = (2048 - self.frequency) * 4;
                self.wave_duty_position = (self.wave_duty_position + 1) & 7;
            }
        }

        // There are four available duty patterns that sets for
        // what part of a period the wave should have high state.
        //
        // 0: 12.5% - 00000001
        // 1: 25%   - 00000011
        // 2: 50%   - 00001111
        // 3: 75%   - 00111111
        //
        // The duty pattern is stored in bit 6-7 of NR11 (NR21)
        let out = WAVE_DUTY[self.duty][self.wave_duty_position as usize];

        // Update sweep at 128 Hz
        if hz128 {
            if let Some(ref mut sweep) = self.sweep {
                sweep.tick_128hz(&mut self.enabled, &mut self.frequency);
            }
        }

        // Update length counter at 256 Hz
        if hz256 && self.length_counter.count_down() {
            self.enabled = false;
        }

        // When envelope steps is non-zero, the envelope (the amplitude)
        // will increase or decrease every (envelope_steps/64) second.

        // Envelope
        if hz64 && self.envelope_periods_initial > 0 {
            if self.envelope_period > 0 {
                self.envelope_period -= 1;

                if self.envelope_period == 0 {
                    self.envelope_period = self.envelope_periods_initial;

                    // Not max volume and volume should increase
                    if self.envelope < 0xF && self.envelope_increasing {
                        self.envelope += 1;
                    }

                    // Not min volume and volume should decrease
                    if self.envelope > 0 && !self.envelope_increasing {
                        self.envelope -= 1;
                    }
                }
            }
        }

        if self.enabled {
            let dac_input = out * self.envelope;
            return self.dac.convert(dac_input);
        }

        0.0
    }
}

pub const CH3_WAVE_LENGTH: usize = 32;

pub struct WaveSoundGenerator {
    // ---------
    // Registers
    // ---------

    // NR30 (0xFF1A): DAC power
    // bit 7:    dac power
    // bit 6..0: not used

    // NR31 (0xFF1B): length load
    // 7..0: load sound length (write only)
    nr31: u8,

    // NR32 (0xFF1C): Volume envelope
    // bit 7, 4..0: not used
    // bit 6..5: volume code (0=0%, 1=100%, 2=50%, 3=25%)
    // nr32: u8,

    // NR33 (0xFF1D): lo bits of frequency
    // - bit 7..0: lo bits of frequency (write only)

    // NR34 (0xFF1E): hi bits of frequency + more
    // - bit 7:    trigger (write only)
    // - bit 6:    length counter enable
    // - bit 5..3: not used
    // - bit 2..0: hi bits of frequency (write only)

    // ---------------
    // Internal values
    // ---------------

    // Frequency. 10 bits. Bit 7..0 in NR13 + bit 9..8 in NR14.
    frequency: u16,

    // Wave pattern containing 32 4-bit samples.
    // These are accessed through 16 registers: 0xFF30 - 0xFF3F.
    // Each register holds two 4-bit samples. The upper bits are
    // played first.
    pub wave: [u8; CH3_WAVE_LENGTH],

    // Internal enabled flag.
    pub enabled: bool,

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value (NR13, NR14) and
    // wave_duty_position moves to next position
    pub frequency_timer: u16,

    pub wave_position: u16,

    // Volume code (0=0%, 1=100%, 2=50%, 3=25%)
    // Bits 6..5 of NR32
    pub volume_code: u8,

    pub length_counter: LengthCounter,
    pub dac: DAC,
    machine: Machine,
}

impl WaveSoundGenerator {
    pub fn new(machine: Machine) -> Self {
        WaveSoundGenerator {
            nr31: 0,
            frequency: 0,

            // The wave is initialized at power-on with some semi-random values.
            // For the DMG, the values below is one possible set.
            // For the CGB, the wave is consistently initialized with the values below.
            wave: match machine {
                Machine::GameBoyDMG | Machine::GameBoyMGB => [
                    0x8, 0x4, 0x4, 0x0, 0x4, 0x3, 0xA, 0xA, 0x2, 0xD, 0x7, 0x8, 0x9, 0x2, 0x3, 0xC,
                    0x6, 0x0, 0x5, 0x9, 0x5, 0x9, 0xB, 0x0, 0x3, 0x4, 0xB, 0x8, 0x2, 0xE, 0xD, 0xA,
                ],
                Machine::GameBoyCGB | Machine::GameBoySGB => [
                    0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF,
                    0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF,
                ],
            },

            length_counter: LengthCounter::new(machine, 256),
            wave_position: 0,
            frequency_timer: 0,
            enabled: false,
            volume_code: 0,
            dac: DAC::new(),
            machine,
        }
    }

    // Reset everything except wave, which is what happens
    // when the sound hardware is powered off by NR52.
    pub fn power_off_reset(&mut self) {
        self.nr31 = 0;
        self.frequency = 0;
        self.length_counter.power_off();
        self.wave_position = 0;
        self.frequency_timer = 0;
        self.enabled = false;
        self.volume_code = 0;
        self.dac = DAC::new();
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            NR30_REG => {
                if self.dac.powered_on {
                    0b1111_1111
                } else {
                    0b0111_1111
                }
            }

            NR31_REG => 0xFF,
            NR32_REG => self.volume_code << 5 | 0b1001_1111,
            NR33_REG => 0xFF,
            NR34_REG => {
                if self.length_counter.is_enabled() {
                    0xFF
                } else {
                    0b1011_1111
                }
            }
            _ => panic!("invalid register in channel 3: {}", address),
        }
    }

    pub fn read_wave_reg(&self, address: usize) -> u8 {
        assert!(address >= 0xFF30 && address <= 0xFF3F);

        // When the wave channel is enabled, accessing any byte in the
        // wave table returns the byte currently being played. On DMG
        // it's even worse, as it only does so for a few clocks after
        // the byte was "played". After that 0xFF is returned instead.
        if self.enabled {
            let adr = self.wave_position as usize / 2;
            return (self.wave[adr * 2] << 4) | self.wave[adr * 2 + 1];
        }

        let adr = address - 0xFF30;
        (self.wave[adr * 2] << 4) | self.wave[adr * 2 + 1]
    }

    pub fn write_reg(&mut self, address: u16, value: u8, seq_step: u8, powered_on: bool) {
        // If unpowered, all writes should be ignored except
        // length value if the machine is original Gameboy DMG
        if !powered_on {
            if matches!(self.machine, Machine::GameBoyDMG) {
                if address == NR31_REG {
                    self.length_counter.write_reg_nrx1(value);
                }
            }
            return;
        }

        match address {
            NR30_REG => {
                self.dac.powered_on = value & 0x80 != 0;
                self.enabled = self.enabled && self.dac.powered_on;
            }
            NR31_REG => {
                self.length_counter.write_reg_nrx1(value);
                self.nr31 = value;
            }
            NR32_REG => self.volume_code = (value & 0b0110_0000) >> 5,
            NR33_REG => self.frequency = (self.frequency & 0b111_0000_0000) | value as u16,
            NR34_REG => {
                self.frequency =
                    (self.frequency & 0b000_1111_1111) | (((value & 0b111) as u16) << 8);

                if self
                    .length_counter
                    .enable(value & 0b0100_0000 != 0, seq_step)
                {
                    self.enabled = false;
                }

                if value & 0x80 != 0 {
                    self.trigger(seq_step);
                }
            }
            _ => panic!("invalid register in channel 3: {}", address),
        }
    }

    pub fn write_wave_reg(&mut self, address: usize, value: u8) {
        let adr = address - 0xFF30;
        self.wave[adr * 2] = (value & 0xF0) >> 4;
        self.wave[adr * 2 + 1] = value & 0x0F
    }

    fn trigger(&mut self, seq_step: u8) {
        self.enabled = true;
        self.length_counter.trigger(256, seq_step);
        self.frequency_timer = (2048 - self.frequency) * 2 + 2;
        self.wave_position = 0;

        // If DAC is not powered on, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update(&mut self, hz256: bool) -> f32 {
        if self.frequency_timer < 4 {
            // If frequency timer reaches 0, reset it to the selected frequency
            // (NR13, NR14) and increment the wave position
            self.frequency_timer += (2048 - self.frequency) * 2 + (self.frequency_timer & 3) - 4;
            self.wave_position = (self.wave_position + 1) & 31;
        } else {
            self.frequency_timer -= 4;
        }

        let mut out = self.wave[self.wave_position as usize];

        // Update length counter at 256 Hz
        if hz256 && self.length_counter.count_down() {
            self.enabled = false;
        }

        out = match self.volume_code {
            0 => 0,
            1 => out,
            2 => out >> 1,
            3 => out >> 2,
            _ => 0,
        };

        if self.enabled {
            return self.dac.convert(out);
        }

        return 0.0;
    }
}

pub struct NoiseSoundGenerator {
    // ---------
    // Registers
    // ---------

    // NR41 (0xFF20): length load
    // 5..0: load sound length (write only)

    // NR42 (0xFF21): Volume envelope
    // bit 7..4: initial volume
    // bit 3:    envelope direction
    // bit 2..0: sweep count

    // NR43 (0xFF22): polynomial counter
    // bit 7..4: shift clock frequency
    // bit 3:    counter step/width (0=15 bits, 1=7 bits)
    // bit 2..0: dividing ratio of frequencies
    nr43: u8,

    // NR44 (0xFF23): counter/consecutive, initial
    // bit 7: initial, 1=restart sound
    // bit 6: counter/consecutive selection (1=stop when length expires)

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value.
    pub frequency_timer: u16,

    // LFSR register. Internal. 15 bits.
    pub lfsr: u16,

    // Polynomial counter. Internal.
    polynomial_counter: u8,

    // Internal register
    pub enabled: bool,

    // Current volume of the envelope filter. Internal register.
    pub envelope: u8,

    // The envelope change period counter. Internal register.
    envelope_period: u8,

    // Period count to initate envelope filter with on trigger.
    // Bits 2..0 of NR12.
    // FIXME: should there *be* a separate start-value,
    // or does the value count down and must be reset
    // before each trigger?
    envelope_periods_initial: u8,

    envelope_increasing: bool,

    // Initial volume of the envelope filter. Bit 7..4 in NR12
    initial_volume: u8,

    pub length_counter: LengthCounter,
    pub dac: DAC,
    machine: Machine,
}

const NOISE_DIVISOR_MAP: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

impl NoiseSoundGenerator {
    pub fn new(machine: Machine) -> Self {
        NoiseSoundGenerator {
            nr43: 0,
            frequency_timer: 0,
            lfsr: 0,
            polynomial_counter: 0,
            enabled: false,
            envelope: 0,
            envelope_period: 0,
            envelope_increasing: false,
            envelope_periods_initial: 0,
            initial_volume: 0,
            length_counter: LengthCounter::new(machine, 64),
            dac: DAC::new(),
            machine,
        }
    }

    pub fn power_off(&mut self) {
        self.nr43 = 0;
        self.frequency_timer = 0;
        self.lfsr = 0;
        self.polynomial_counter = 0;
        self.enabled = false;
        self.envelope = 0;
        self.envelope_period = 0;
        self.envelope_increasing = false;
        self.envelope_periods_initial = 0;
        self.initial_volume = 0;
        self.length_counter.power_off();
        self.dac = DAC::new();
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            NR40_REG => 0xFF,
            NR41_REG => 0xFF,
            NR42_REG => {
                let nr42 = (self.initial_volume << 4) | self.envelope_periods_initial;
                if self.envelope_increasing {
                    nr42 | 0b0000_1000
                } else {
                    nr42
                }
            }
            NR43_REG => self.nr43,
            NR44_REG => {
                if self.length_counter.is_enabled() {
                    0b1111_1111
                } else {
                    0b1011_1111
                }
            }
            _ => panic!(
                "Invalid register in noise sound generator: 0x{:04x}",
                address
            ),
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8, seq_step: u8, powered_on: bool) {
        // If unpowered, all writes should be ignored except
        // length value if the machine is original Gameboy DMG
        if !powered_on {
            if matches!(self.machine, Machine::GameBoyDMG) {
                if address == NR41_REG {
                    self.length_counter.write_reg_nrx1(value);
                }
            }
            return;
        }

        match address {
            NR40_REG => {}
            NR41_REG => self.length_counter.write_reg_nrx1(value),
            NR42_REG => {
                self.initial_volume = (value >> 4) & 0xF;
                self.dac.powered_on = value & 0b1111_1000 != 0;
                self.enabled = self.enabled && self.dac.powered_on;
                self.envelope_increasing = (value & 0b1000) != 0;
                self.envelope_periods_initial = value & 0b111;
            }
            NR43_REG => self.nr43 = value,
            NR44_REG => {
                if self
                    .length_counter
                    .enable(value & 0b0100_0000 != 0, seq_step)
                {
                    self.enabled = false;
                }

                if value & 0b1000_0000 != 0 {
                    self.trigger(seq_step);
                }
            }
            _ => panic!("invalid register {}", address),
        }
    }

    pub fn trigger(&mut self, seq_step: u8) {
        self.enabled = true;
        self.length_counter.trigger(64, seq_step);
        self.lfsr = 0b0111_1111_1111_1111;

        let divisor_code = self.nr43 & 7;
        let divisor = NOISE_DIVISOR_MAP[divisor_code as usize];
        let shift_amount = (self.nr43 & 0xF0) >> 4;
        self.frequency_timer = (divisor as u16) << (shift_amount as u16);

        self.envelope = self.initial_volume;
        self.envelope_period = self.envelope_periods_initial;

        // If DAC is not powered, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update(&mut self, hz64: bool, hz256: bool) -> f32 {
        // Decrement frequency timer
        if self.frequency_timer >= 4 {
            self.frequency_timer -= 4;

            if self.frequency_timer == 0 {
                let divisor_code = self.nr43 & 7;
                let divisor = NOISE_DIVISOR_MAP[divisor_code as usize];
                let shift_amount = (self.nr43 & 0xF0) >> 4;
                self.frequency_timer = (divisor as u16) << (shift_amount as u16);

                let xor_result = (self.lfsr & 1) ^ ((self.lfsr & 2) >> 1);
                self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

                if self.nr43 & 0b1000 != 0 {
                    self.lfsr &= !(1 << 6);
                    self.lfsr |= xor_result << 6;
                }
            }
        }

        // Update length counter at 256 Hz
        if hz256 && self.length_counter.count_down() {
            self.enabled = false;
        }

        // When envelope steps is non-zero, the envelope (the amplitude)
        // will increase or decrease every (envelope_steps/64) second.

        // Envelope
        if hz64 && self.envelope_periods_initial > 0 {
            if self.envelope_period > 0 {
                self.envelope_period -= 1;

                if self.envelope_period == 0 {
                    self.envelope_period = self.envelope_periods_initial;

                    // Not max volume and volume should increase
                    if self.envelope < 0xF && self.envelope_increasing {
                        self.envelope += 1;
                    }

                    // Not min volume and volume should decrease
                    if self.envelope > 0 && !self.envelope_increasing {
                        self.envelope -= 1;
                    }
                }
            }
        }

        if self.enabled {
            let out = if self.lfsr & 1 == 0 { 0 } else { 1 };
            let dac_input = out * self.envelope;
            return self.dac.convert(dac_input);
        }

        0.0
    }
}

pub trait AudioRecorder {
    fn mono(&mut self, sample: f32);
    fn gen1(&mut self, sample: f32);
    fn gen2(&mut self, sample: f32);
    fn flush(&mut self);
}

pub struct AudioProcessingUnit {
    machine: Machine,

    pub s1: SquareWaveSoundGenerator,
    pub s2: SquareWaveSoundGenerator,
    pub ch3: WaveSoundGenerator,
    pub ch4: NoiseSoundGenerator,
    pub nr50: u8,
    pub nr51: u8,

    // Bit 7 of NR52. Controls power to the audio hardware
    pub powered_on: bool,

    // Producer for the output ring buffer.
    // Every cycle one sample is appended to this buffer.
    pub buf: Option<Producer<f32>>,

    pub recorder: Option<Box<dyn AudioRecorder>>,
}

impl AudioProcessingUnit {
    pub fn new(machine: Machine) -> Self {
        AudioProcessingUnit {
            machine,
            s1: SquareWaveSoundGenerator::new(true, machine),
            s2: SquareWaveSoundGenerator::new(false, machine),
            ch3: WaveSoundGenerator::new(machine),
            ch4: NoiseSoundGenerator::new(machine),
            nr50: 0,
            nr51: 0,
            buf: None,
            recorder: None,
            powered_on: false,
        }
    }

    // Perform a complete reset. Used when resetting the whole machine.
    // Note that the APU can't easily be recreated, as it has a ringbuf
    // producer that can't be moved to a new instance of it, so instead
    // we must reset all values.
    pub fn reset(&mut self) {
        self.s1 = SquareWaveSoundGenerator::new(true, self.machine);
        self.s2 = SquareWaveSoundGenerator::new(false, self.machine);
        self.ch3 = WaveSoundGenerator::new(self.machine);
        self.ch4 = NoiseSoundGenerator::new(self.machine);
        self.nr50 = 0;
        self.nr51 = 0;
        self.powered_on = false;
    }

    pub fn seq_step(&self, div_counter: u16) -> u8 {
        return (div_counter >> 13) as u8;
    }

    pub fn update(&mut self, div_counter: u16) {
        // NR52 bit 7 is used to disable the sound system completely

        // The Frame Sequencer is used to generate clocks as 256 Hz,
        // 128 Hz and 64 Hz. See this table copied from gbdev wiki:
        //
        // Step   Length Ctr  Vol Env     Sweep
        // ---------------------------------------
        // 0      Clock       -           -
        // 1      -           -           -
        // 2      Clock       -           Clock
        // 3      -           -           -
        // 4      Clock       -           -
        // 5      -           -           -
        // 6      Clock       -           Clock
        // 7      -           Clock       -
        // ---------------------------------------
        // Rate   256 Hz      64 Hz       128 Hz
        //
        // To allow for all these to be generated, the frame sequencer
        // must tick at 512 Hz (every 8192'th cycle). Since the pattern
        // repeats every 8'th time, we can initialize frame sequencer to
        // 65536 (8192 * 512), decrement to zero and wrap around.
        //
        // The frame sequencer is based on the DIV timer. DIV is the top
        // 8 bits of the 16-bit timer that decrements for every clock cycle.
        //
        // Note that for CGB, div_counter should be shifted 14 bits instead of 13
        // as the DIV registers decrements at double speed. That means only two
        // bits remain, so we must have another strategy for the 64 Hz clock.
        let mut hz64 = false;
        let mut hz128 = false;
        let mut hz256 = false;
        if div_counter % 8192 == 0 {
            let step = self.seq_step(div_counter);
            hz64 = step == 7;
            hz128 = step == 2 || step == 6;
            hz256 = step & 1 == 0;
        }

        let ch1_output = self.s1.update(hz64, hz128, hz256);
        let ch2_output = self.s2.update(hz64, hz128, hz256);
        let ch3_output = self.ch3.update(hz256);
        let ch4_output = self.ch4.update(hz64, hz256);

        let sample = (ch1_output + ch2_output + ch3_output + ch4_output) as f32;

        if let Some(ref mut rec) = self.recorder {
            rec.gen1(ch1_output as f32);
            rec.gen2(ch2_output as f32);
            rec.mono(sample);
        }

        if let Some(ref mut producer) = self.buf {
            if producer.is_full() {
                eprintln!("Buffer is full {} {}", producer.capacity(), producer.len());
                return;
            }

            producer
                .push(sample as f32)
                .expect("Failed to push sample to audio buffer");
        }
    }

    pub fn read_nr52(&self) -> u8 {
        let mut nr52: u8 = 0;
        if self.powered_on {
            nr52 = 0x80;
        }
        if self.s1.enabled {
            nr52 |= 0b0001;
        }
        if self.s2.enabled {
            nr52 |= 0b0010;
        }
        if self.ch3.enabled {
            nr52 |= 0b0100;
        }
        if self.ch4.enabled {
            nr52 |= 0b1000;
        }
        nr52
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0xFF10..=0xFF14 => self.s1.read_reg(address),
            0xFF15..=0xFF19 => self.s2.read_reg(address),
            0xFF1A..=0xFF1E => self.ch3.read_reg(address),
            0xFF1F..=0xFF23 => self.ch4.read_reg(address),
            NR50_REG => self.nr50,
            NR51_REG => self.nr51,
            NR52_REG => self.read_nr52() | 0b0111_0000,
            0xFF27..=0xFF2F => 0xFF,
            0xFF30..=0xFF3F => self.ch3.read_wave_reg(address as usize),
            _ => 0,
        }
    }

    fn power_on(&mut self) {
        if !self.powered_on {
            self.powered_on = true;
        }
    }

    fn power_off(&mut self) {
        if self.powered_on {
            self.powered_on = false;
            self.s1.power_off();
            self.s2.power_off();
            self.ch3.power_off_reset();
            self.ch4.power_off();
            self.nr50 = 0;
            self.nr51 = 0;
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8, div_counter: u16) {
        // Writes to NR52 and the wave memory allways work, even
        // when the sound hardware is powered off.
        match address {
            NR52_REG => {
                if value & 0x80 != 0 {
                    self.power_on();
                } else {
                    self.power_off();
                }
            }
            0xFF30..=0xFF3F => self.ch3.write_wave_reg(address as usize, value),
            _ => {}
        }

        match address {
            0xFF10..=0xFF14 => {
                self.s1
                    .write_reg(address, value, self.seq_step(div_counter), self.powered_on)
            }
            0xFF15..=0xFF19 => {
                self.s2
                    .write_reg(address, value, self.seq_step(div_counter), self.powered_on)
            }
            0xFF1A..=0xFF1E => {
                self.ch3
                    .write_reg(address, value, self.seq_step(div_counter), self.powered_on)
            }
            0xFF1F => {}
            0xFF20..=0xFF23 => {
                self.ch4
                    .write_reg(address, value, self.seq_step(div_counter), self.powered_on)
            }
            NR50_REG => {
                if self.powered_on {
                    self.nr50 = value
                }
            }
            NR51_REG => {
                if self.powered_on {
                    self.nr51 = value
                }
            }
            0xFF27..=0xFF2F => {}
            _ => {}
        }
    }
}

fn reg_name(address: u16) -> String {
    match address {
        0xFF10..=0xFF14 => format!("NR{:02X}", address - 0xFF10 + 0x10),
        0xFF15..=0xFF19 => format!("NR{:02X}", address - 0xFF15 + 0x20),
        0xFF1A..=0xFF1E => format!("NR{:02X}", address - 0xFF1A + 0x30),
        0xFF1F..=0xFF23 => format!("NR{:02X}", address - 0xFF1F + 0x40),
        _ => format!("{:04X}", address),
    }
}
