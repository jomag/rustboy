use crate::apu::dac::DAC;
use crate::apu::length_counter::LengthCounter;
use crate::emu::Machine;
use crate::mmu::{NR40_REG, NR41_REG, NR42_REG, NR43_REG, NR44_REG};

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

    pub fn update_4t(&mut self, hz64: bool, hz256: bool) -> i16 {
        assert!(self.frequency_timer % 4 == 0);

        // Decrement frequency timer
        if self.frequency_timer <= 4 {
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
        } else {
            self.frequency_timer -= 4;
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

        0
    }
}
