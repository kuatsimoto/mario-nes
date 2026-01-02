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
    iNES2,
    iNESArch,
}
#[derive(Debug)]
struct Cartridge {
    //Cartridge struct used to define all flags and parameters in NES ROM header.
    //This struct will also store the raw bytes of the header and the PRG/CHR ROM
    validated: bool,
    prg_rom_size: usize,
    chr_rom_size: usize,
    nametable_mirroring: Nametable, //This can be an enum?
    prg_ram_flag: bool,
    trainer_flag: bool,
    alt_nametable_flag: bool,
    lower_mapper_nybble: u8,
    vs_unisystem: bool,
    upper_mapper_nybble: u8,
    prg_ram_size: u8,
    nes_mode: NESMode,
}
impl Cartridge {
    fn load(bytes: &[u8]) -> Self {
        //Need to ingest bytes and put them into Self
        
        //Validate header
        let validated = &bytes[0..4] == b"NES\x1A";
        let prg_rom_size = bytes[4] as usize;
        let chr_rom_size = bytes[5] as usize;

        let nametable_mirroring = (bytes[6] & 0x01) != 0;
        let nametable_mirroring = match nametable_mirroring {
            true => Nametable::Vertical,
            false => Nametable::Horizontal,
        };
        
        //Flags 6
        let prg_ram_flag = (bytes[6] & 0x02)!=0; //second bit
        let trainer_flag = (bytes[6] & 0x04)!=0; //third bit
        let alt_nametable_flag = (bytes[6] & 0x08)!=0; //fourth bit
        let lower_mapper_nybble = (bytes[6] & 0xF0)>>4; //last byte
        
        //Flags 7
        let vs_unisystem = (bytes[7] & 0x01) != 0;
        let upper_mapper_nybble = (bytes[7] & 0xF0)>>4;

        //Flags 8
        let prg_ram_size = bytes[8];

        //Detect iNES mode
        let mut nes_mode = NESMode::iNES;
        let ines_compare = bytes[7] & 0x0C;
        let file_size = bytes.len();
        let mut trainer_size = 0usize;

        if trainer_flag {
            trainer_size = 512;
        }

        if ines_compare == 0 && bytes[12..16].iter().all(|&x| x == 0){
            nes_mode = NESMode::iNES;
        }
        if ines_compare == 0x04{
            nes_mode = NESMode::iNESArch;
        }
        if ines_compare == 0x08{
            let prg_banks = (bytes[4] | (bytes[9] & 0x0F)) as usize;
            let chr_banks = (bytes[5] | ((bytes[9] & 0xF0) >> 4)) as usize;
            let expected_size = 16 + trainer_size + prg_banks + chr_banks;
            //Compare expected size with actual file size
        }

        Self {
            validated,
            prg_rom_size,
            chr_rom_size,
            nametable_mirroring,
            prg_ram_flag,
            trainer_flag,
            alt_nametable_flag,
            lower_mapper_nybble,
            vs_unisystem,
            upper_mapper_nybble,
            prg_ram_size,
            nes_mode,
        }
    }
}

pub fn read_file() -> std::io::Result<()> {
    //Read rom file into memory
    let bytes = fs::read("nestest.nes").expect("Failed to read ROM");

    // for byte in bytes.iter() {
    //     println!("{:02X}", byte);
    // }

    // assert_eq!(&bytes[0..4], b"NES\x1A");
    println!("{:X?}", &bytes[0..16]);
    let cartridge = Cartridge::load(&bytes);
    println!("{:#?}",cartridge);
    // println!("Valid iNES header detected");

    Ok(())
}
