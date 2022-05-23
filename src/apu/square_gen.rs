use crate::apu::dac::DAC;
use crate::apu::length_counter::LengthCounter;
use crate::apu::sweep::Sweep;
use crate::emu::Machine;
use crate::mmu::{
    NR10_REG, NR11_REG, NR12_REG, NR13_REG, NR14_REG, NR20_REG, NR21_REG, NR22_REG, NR23_REG,
    NR24_REG,
};

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
    pub frequency: u16,

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

    pub fn read_reg(&self, address: usize) -> u8 {
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

    pub fn write_reg(&mut self, address: usize, value: u8, seq_step: u8, powered_on: bool) {
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
                self.frequency = (self.frequency & 0b111_0000_0000) | value as u16
            }
            NR14_REG | NR24_REG => {
                self.frequency =
                    (self.frequency & 0b000_1111_1111) | (((value & 0b111) as u16) << 8);

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

        // If DAC is not powered, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update_4t(&mut self, hz64: bool, hz128: bool, hz256: bool) -> i16 {
        assert!(self.frequency_timer % 4 == 0);

        // Decrement frequency timer
        // FIXME: Handle frequency timer being less than 4.
        //        When so, add to the frequency timer instead:
        //        `if (tmr <= 4) { tmr += freq } else { tmr -= 4 }`
        if self.frequency_timer <= 4 {
            // If frequency timer reaches 0, reset it to the selected frequency
            // (NR13, NR14) and increment the wave duty position
            self.frequency_timer = (2048 - self.frequency) * 4;
            self.wave_duty_position = (self.wave_duty_position + 1) & 7;
        } else {
            self.frequency_timer -= 4;
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

        0
    }
}
