use std::fs;

//Define an enum for the Nametable option
#[derive(Debug)]
enum Nametable {
    Horizontal,
    Vertical,
}
#[derive(Debug)]
#[allow(non_camel_case_types)]
enum NESMode {
    iNES,
    NES2,
    iNESArch,
}
#[derive(Debug)]
pub struct Cartridge {
    //Cartridge struct used to define all flags and parameters in NES ROM header.
    //This struct will also store the raw bytes of the header and the PRG/CHR ROM
    validated: bool,
    prg_rom_banks: usize,
    chr_rom_banks: usize,
    prg_size_bytes: usize,
    chr_size_bytes: usize,
    nametable_mirroring: Nametable, //This can be an enum?
    has_battery: bool,
    trainer_flag: bool,
    alt_nametable_flag: bool,
    mapper: u8,
    vs_unisystem: bool,
    prg_ram_size_bytes: usize,
    nes_mode: NESMode,
    raw_header_bytes: Vec<u8>,
    prg_rom_data: Vec<u8>,
    chr_rom_data: Vec<u8>,
    chr_ram_data: Vec<u8>,
}
impl Cartridge {
    fn load(bytes: &[u8]) -> Result<Self, &'static str> {
        //Need to ingest bytes and put them into Self

        if bytes.len() < 16 {
            return Err("File size too small, invalid header");
        }

        //Validate header
        let validated = &bytes[0..4] == b"NES\x1A";
        if !validated {
            return Err("Invalid ROM header");
        }

        let nametable_mirroring = (bytes[6] & 0x01) != 0;
        let nametable_mirroring = match nametable_mirroring {
            true => Nametable::Vertical,
            false => Nametable::Horizontal,
        };

        let mut prg_rom_banks = bytes[4] as usize;
        let mut chr_rom_banks = bytes[5] as usize;
        let mut prg_size_bytes = prg_rom_banks * 16 * 1024;
        let mut chr_size_bytes = chr_rom_banks * 8 * 1024;

        //Flags 6
        let has_battery = (bytes[6] & 0x02) != 0; //second bit
        let trainer_flag = (bytes[6] & 0x04) != 0; //third bit
        let alt_nametable_flag = (bytes[6] & 0x08) != 0; //fourth bit
        let lower_mapper_nybble = (bytes[6] & 0xF0) >> 4; //last byte

        //Flags 7
        let vs_unisystem = (bytes[7] & 0x01) != 0;
        let upper_mapper_nybble = (bytes[7] & 0xF0) >> 4;

        //Flags 8
        let prg_ram_banks = bytes[8] as usize;
        let prg_ram_size_bytes: usize;
        if prg_ram_banks == 0 {
            prg_ram_size_bytes = 8 * 1024;
        } else {
            prg_ram_size_bytes = prg_ram_banks * 8 * 1024;
        }

        //Calculate Mapper
        let mapper = (upper_mapper_nybble << 4) | (lower_mapper_nybble);

        //Detect iNES mode
        let nes_mode: NESMode;
        let ines_compare = bytes[7] & 0x0C;
        let file_size = bytes.len();
        let mut trainer_size = 0usize;

        if trainer_flag {
            trainer_size = 512;
        }

        if ines_compare == 0 && bytes[12..16].iter().all(|&x| x == 0) {
            nes_mode = NESMode::iNES;
        } else if ines_compare == 0x04 {
            nes_mode = NESMode::iNESArch;
        } else if ines_compare == 0x08 {
            prg_rom_banks = (bytes[4] as u16 | ((bytes[9] as u16 & 0x000F) << 8)) as usize;
            chr_rom_banks = (bytes[5] as u16 | (bytes[9] as u16 & 0x00F0) << 4) as usize;
            prg_size_bytes = prg_rom_banks * 16 * 1024;
            chr_size_bytes = chr_rom_banks * 8 * 1024;
            let expected_size = 16 + trainer_size + prg_size_bytes + chr_size_bytes;
            //Compare expected size with actual file size
            if file_size >= expected_size {
                nes_mode = NESMode::NES2;
            } else {
                return Err("NES 2.0 header detected but file size is too small.");
            }
        } else {
            return Err("Invalid NES Mode");
        }

        let raw_header_bytes = bytes[0..16].to_owned();

        //Slice PRG ROM, CHR ROM/CHR RAM from byte array and store as fields
        let prg_offset = 16 + trainer_size;
        let chr_offset = prg_offset + prg_size_bytes;

        //Check if PRG ROM fits in file
        if prg_offset + prg_size_bytes > bytes.len() {
            return Err("ROM too small for PRG data");
        }
        let prg_rom_data = bytes[prg_offset..prg_size_bytes + prg_offset].to_vec();

        let mut chr_rom_data = vec![];
        let mut chr_ram_data = vec![];

        //Check if CHR data fits in ROM
        if chr_rom_banks == 0 {
            chr_ram_data = vec![0; 8 * 1024]
        } else {
            if chr_offset + chr_size_bytes > bytes.len() {
                return Err("ROM too small for CHR data");
            }
            chr_rom_data = bytes[chr_offset..chr_size_bytes + chr_offset].to_vec();
        }
        Ok(Self {
            validated,
            prg_rom_banks,
            chr_rom_banks,
            prg_size_bytes,
            chr_size_bytes,
            nametable_mirroring,
            has_battery,
            trainer_flag,
            alt_nametable_flag,
            mapper,
            vs_unisystem,
            prg_ram_size_bytes,
            nes_mode,
            raw_header_bytes,
            prg_rom_data,
            chr_rom_data,
            chr_ram_data,
        })
    }
    // fn new() -> Self {
    //     Self {
    //         validated: false,
    //         prg_rom_banks: 0,
    //         chr_rom_banks: 0,
    //         prg_size_bytes: 0,
    //         chr_size_bytes: 0,
    //         nametable_mirroring: Nametable::Horizontal,
    //         has_battery: false,
    //         trainer_flag: false,
    //         alt_nametable_flag: false,
    //         mapper: 0,
    //         vs_unisystem: false,
    //         prg_ram_size_bytes: 0,
    //         nes_mode: NESMode::iNES,
    //         raw_header_bytes: Vec::new(),
    //         prg_rom_data: Vec::new(),
    //         chr_rom_data: Vec::new(),
    //         chr_ram_data: Vec::new(),
    //     }
    // }
}

//Eventually, this will take file name as an argument to allow for multiple ROMs to load
pub fn load_rom() -> Result<Cartridge, &'static str> {
    //Read rom file into memory
    let bytes = fs::read("nestest.nes").expect("Failed to read ROM");

    let cartridge = Cartridge::load(&bytes)?;
    Ok(cartridge)
}
