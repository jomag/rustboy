pub const ROM_BANK_SIZE: usize = 16384;
pub const RAM_BANK_SIZE: usize = 8192;

pub struct CartridgeHeader {
    pub licensee_code: [u8; 2],
    pub old_licensee_code: u8,
    pub checksum: u8,
    pub global_checksum: u16,
    pub sgb_features: bool,
    pub cartridge_type: u8,
    pub rom_bank_count: usize,
    pub rom_size: usize,
    pub ram_bank_count: usize,
    pub ram_size: usize,
}

impl CartridgeHeader {
    pub fn from_header(header: &Vec<u8>) -> Self {
        let licensee_code: [u8; 2] = [header[0x144], header[0x145]];

        let rom_bank_count = match header[0x148] {
            0..=8 => 2 << header[0x148],
            _ => 0,
        };

        let ram_bank_count = match header[0x0149] {
            0 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => 0,
        };

        CartridgeHeader {
            licensee_code,
            old_licensee_code: header[0x14B],
            checksum: header[0x14D],
            global_checksum: ((header[0x14E] as u16) << 8) | header[0x14F] as u16,
            sgb_features: header[0x146] == 0x03,
            cartridge_type: header[0x147],
            rom_bank_count,
            ram_bank_count,
            rom_size: rom_bank_count * ROM_BANK_SIZE,
            ram_size: ram_bank_count * RAM_BANK_SIZE,
        }
    }

    pub fn licensee(&self) -> String {
        match std::str::from_utf8(&self.licensee_code) {
            Ok("00") => "None",
            Ok("01") => "Nintendo R&D1",
            Ok("08") => "Capcom",
            Ok("13") => "Electronic Arts",
            Ok("18") => "Hudson Soft",
            Ok("19") => "b-ai",
            Ok("20") => "kss",
            Ok("22") => "pow",
            Ok("24") => "PCM Complete",
            Ok("25") => "san-x",
            Ok("28") => "Kemco Japan",
            Ok("29") => "seta",
            Ok("30") => "Viacom",
            Ok("31") => "Nintendo",
            Ok("32") => "Bandai",
            Ok("33") => "Ocean/Acclaim",
            Ok("34") => "Konami",
            Ok("35") => "Hector",
            Ok("37") => "Taito",
            Ok("38") => "Hudson",
            Ok("39") => "Banpresto",
            Ok("41") => "Ubi Soft",
            Ok("42") => "Atlus",
            Ok("44") => "Malibu",
            Ok("46") => "angel",
            Ok("47") => "Bullet-Proof",
            Ok("49") => "irem",
            Ok("50") => "Absolute",
            Ok("51") => "Acclaim",
            Ok("52") => "Activision",
            Ok("53") => "American sammy",
            Ok("54") => "Konami",
            Ok("55") => "Hi tech entertainment",
            Ok("56") => "LJN",
            Ok("57") => "Matchbox",
            Ok("58") => "Mattel",
            Ok("59") => "Milton Bradley",
            Ok("60") => "Titus",
            Ok("61") => "Virgin",
            Ok("64") => "LucasArts",
            Ok("67") => "Ocean",
            Ok("69") => "Electronic Arts",
            Ok("70") => "Infogrames",
            Ok("71") => "Interplay",
            Ok("72") => "Broderbund",
            Ok("73") => "sculptured",
            Ok("75") => "sci",
            Ok("78") => "THQ",
            Ok("79") => "Accolade",
            Ok("80") => "misawa",
            Ok("83") => "lozc",
            Ok("86") => "Tokuma Shoten Intermedia",
            Ok("87") => "Tsukuda Original",
            Ok("91") => "Chunsoft",
            Ok("92") => "Video system",
            Ok("93") => "Ocean/Acclaim",
            Ok("95") => "Varie",
            Ok("96") => "Yonezawa/s’pal",
            Ok("97") => "Kaneko",
            Ok("99") => "Pack in soft",
            Ok("A4") => "Konami (Yu-Gi-Oh!)",
            Ok(_) => "Unknown",
            Err(_) => "Unknown",
        }
        .to_string()
    }
}
