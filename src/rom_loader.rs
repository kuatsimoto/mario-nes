use std::fs;

//Define an enum for the Nametable option
enum Nametable {
    Horizontal,
    Vertical,
}

struct Cartridge {
    //Cartridge struct used to define all flags and parameters in NES ROM header.
    //This struct will also store the raw bytes of the header and the PRG/CHR ROM
    validated: bool,
    prg_rom_size: u8,
    chr_rom_size: u8,
    nametable_arr: Nametable, //This can be an enum?
    prg_ram_flag: bool,
    trainer_flag: bool,
    alt_nametable_flag: bool,
    lower_mapper_nybble: u8,
    vs_unisystem: bool,
    upper_mapper_nybble: u8,
    prg_ram_size: u8,
}

pub fn read_file() -> std::io::Result<()> {
    //Read rom file into memory
    let bytes = fs::read("nestest.nes").expect("Failed to read ROM");

    // for byte in bytes.iter() {
    //     println!("{:02X}", byte);
    // }

    assert_eq!(&bytes[0..4], b"NES\x1A");
    println!("Valid iNES header detected");

    Ok(())
}
