use super::super::emu::Machine;
use super::super::mmu::{NR30_REG, NR31_REG, NR32_REG, NR33_REG, NR34_REG};
use super::dac::DAC;
use super::length_counter::LengthCounter;

pub const CH3_WAVE_MEMORY_SIZE: usize = 16;

pub struct WaveSoundGenerator {
    // ---------
    // Registers
    // ---------

    // NR30 (0xFF1A): DAC power
    // bit 7:    dac power
    // bit 6..0: not used

    // NR31 (0xFF1B): length load
    // 7..0: load sound length (write only)

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
    pub wave: [u8; CH3_WAVE_MEMORY_SIZE],

    // Internal enabled flag.
    pub enabled: bool,

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value (NR13, NR14) and
    // wave_duty_position moves to next position
    pub frequency_timer: i16,

    pub wave_position: u16,

    // Volume code (0=0%, 1=100%, 2=50%, 3=25%)
    // Bits 6..5 of NR32
    pub volume_code: u8,

    // Obscure behavior in DMG:
    // When channel is enabled and a sample has just been
    // read, the value of that sample will be returned on any
    // access to the wave memory. Shortly thereafter (2 cycles?)
    // the DMG will return 0xFF for every read.
    // The CGB works the same, except it always returns the
    // last sample read. Test: "09-wave read while on"
    wave_recently_read: bool,

    // The sample currently being played
    sample_buffer: u8,

    pub length_counter: LengthCounter,
    pub dac: DAC,
    machine: Machine,
}

impl WaveSoundGenerator {
    pub fn new(machine: Machine) -> Self {
        WaveSoundGenerator {
            frequency: 0,

            // The wave is initialized at power-on with some semi-random values.
            // For the DMG, the values below is one possible set.
            // For the CGB, the wave is consistently initialized with the values below.
            wave: match machine {
                Machine::GameBoyDMG | Machine::GameBoyMGB => [
                    0x84, 0x40, 0x43, 0xAA, 0x2D, 0x78, 0x92, 0x3C, 0x60, 0x59, 0x59, 0xB0, 0x34,
                    0xB8, 0x2E, 0xDA,
                ],
                Machine::GameBoyCGB | Machine::GameBoySGB => [
                    0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
                    0xFF, 0x00, 0xFF,
                ],
            },

            length_counter: LengthCounter::new(machine, 256),
            wave_position: 0,
            frequency_timer: 0,
            enabled: false,
            volume_code: 0,
            dac: DAC::new(),
            wave_recently_read: false,
            sample_buffer: 0,
            machine,
        }
    }

    // Reset everything except wave, which is what happens
    // when the sound hardware is powered off by NR52.
    pub fn power_off_reset(&mut self) {
        self.frequency = 0;
        self.length_counter.power_off();
        self.wave_position = 0;
        self.frequency_timer = 0;
        self.enabled = false;
        self.volume_code = 0;
        self.sample_buffer = 0;
        self.dac = DAC::new();
    }

    pub fn read_reg(&self, address: usize) -> u8 {
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
            if matches!(self.machine, Machine::GameBoyDMG) {
                if !self.wave_recently_read {
                    return 0xFF;
                }
            }

            return self.sample_buffer;
        }

        self.wave[address - 0xFF30]
    }

    pub fn write_reg(&mut self, address: usize, value: u8, seq_step: u8, powered_on: bool) {
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
            NR31_REG => self.length_counter.write_reg_nrx1(value),
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

    // Get the 4-bit sample at position n without any side effects
    pub fn get_sample(&self, n: usize) -> u8 {
        if n & 1 == 0 {
            (self.wave[n / 2] >> 4) & 0xF
        } else {
            self.wave[n / 2] & 0xF
        }
    }

    pub fn write_wave_reg(&mut self, address: usize, value: u8) {
        if self.enabled {
            if matches!(self.machine, Machine::GameBoyDMG) {
                if !self.wave_recently_read {
                    return;
                }
            }

            let adr = self.wave_position / 2;
            self.wave[adr as usize] = value;
            return;
        }

        let adr = address - 0xFF30;
        self.wave[adr] = value;
    }

    fn trigger(&mut self, seq_step: u8) {
        match self.machine {
            Machine::GameBoyDMG => {
                if self.enabled && self.frequency_timer <= 2 && self.dac.powered_on {
                    let byte_pos = (self.wave_position + 1) as usize / 2;
                    if byte_pos < 4 {
                        self.wave[0] = self.wave[byte_pos];
                    } else {
                        let src = byte_pos & 0xC;
                        self.wave[0] = self.wave[src];
                        self.wave[1] = self.wave[src + 1];
                        self.wave[2] = self.wave[src + 2];
                        self.wave[3] = self.wave[src + 3];
                    }
                }
            }
            _ => {}
        }

        self.enabled = true;
        self.length_counter.trigger(256, seq_step);
        self.frequency_timer = (2048 - self.frequency as i16) * 2 + 6;

        self.wave_position = 0;
        self.sample_buffer = self.wave[self.wave_position as usize / 2];

        // If DAC is not powered on, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update_4t(&mut self, hz256: bool) -> i16 {
        if self.frequency_timer <= 4 {
            // Handle obscure behavior in DMG
            if self.frequency_timer == 4 {
                self.wave_recently_read = true;
            }

            // If frequency timer reaches 0, reset it to the selected frequency
            // (NR13, NR14) and increment the wave position
            self.frequency_timer += (2048 - self.frequency as i16) * 2 - 4;
            self.wave_position = (self.wave_position + 1) & 31;
            self.sample_buffer = self.wave[self.wave_position as usize / 2]
        } else {
            self.frequency_timer -= 4;
            self.wave_recently_read = false;
        }

        let mut out = if self.wave_position & 1 == 0 {
            self.sample_buffer >> 4 & 0xF
        } else {
            self.sample_buffer & 0xF
        };

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

        0
    }
}
