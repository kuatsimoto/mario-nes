use std::fs;
// use crate::rom_loader::Cartridge;

pub struct Mapper {
    mapper: usize,
    //For later
    //mapper_state: usize,
}
impl Mapper {
    fn new(mapper: usize) -> Self {
        Self { mapper }
    }
    fn cpu_read(&self, prg_rom: &[u8], prg_ram: &[u8], address: u16) -> u8 {
        //This will eventually be a switch statement for all implemented mappers
        match self.mapper {
            0 => self.cpu_read_mapper_0(prg_rom, prg_ram, address),
            _ => 0,
        }
    }
    fn cpu_write(&self, prg_ram: &mut [u8], address: u16, value: u8) {
        match self.mapper{
            0 => self.cpu_write_mapper_0(prg_ram, address, value),
            _ => (),
        }

    }
    fn cpu_read_mapper_0(&self, prg_rom: &[u8], prg_ram: &[u8], address: u16) -> u8 {
        //CPU $6000-$7FFF: Unbanked PRG-RAM, mirrored as necessary to fill entire 8 KiB window, write protectable with an external switch.
        //CPU $8000-$BFFF: First 16 KiB of PRG-ROM.
        //CPU $C000-$FFFF: Last 16 KiB of PRG-ROM (NROM-256) or mirror of $8000-$BFFF (NROM-128).
        //PPU $0000-$1FFF: 8 KiB CHR-ROM.

        //Need a sanity check here. Check if address is reasonable

        let translated_address: usize;

        //Get size of PRG_ROM and RAM
        let prg_rom_size: usize = prg_rom.len();
        let prg_ram_size: usize = prg_ram.len();

        //Notes for my own sake:
        //(address - base) % bank_size gives an automatic translated address with baked-in mirroring
        //All addresses up to bank_size are mirrored to the next bytes of bank_size

        //Start address mapping
        match address {
            0x6000..=0x7FFF => {
                translated_address = (address as usize - 0x6000) % prg_ram_size;
                prg_ram[translated_address]
            }
            0x8000..=0xFFFF => {
                translated_address = (address as usize - 0x8000) % prg_rom_size;
                prg_rom[translated_address]
            }
            _ => {
                println!("Invalid address");
                0 as u8
            }
        }
    }
    fn cpu_write_mapper_0(&self, prg_ram: &mut [u8], address: u16, value: u8) {

        let translated_address: usize;

        //Get size of PRG_RAM
        let prg_ram_size: usize = prg_ram.len();

        //Start address matching. Need to mutate prg_ram location to new value
        match address {
            0x6000..=0x7FFF => {
                translated_address = (address as usize - 0x6000) % prg_ram_size;
                prg_ram[translated_address] = value;
            },
            _ => {
                println!("Invalid address")
            }
        }
    }
}
