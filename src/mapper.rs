use std::fs;
// use crate::rom_loader::Cartridge;

pub struct Mapper {
    mapper: usize,
    //For later
    //mapper_state: usize,
}
impl Mapper {
    pub fn new(mapper: usize) -> Self {
        Self { mapper }
    }
    pub fn cpu_read(&self, prg_rom: &[u8], prg_ram: &[u8], address: u16) -> u8 {
        //This will eventually be a switch statement for all implemented mappers
        match self.mapper {
            0 => self.cpu_read_mapper_0(prg_rom, prg_ram, address),
            _ => 0,
        }
    }
    pub fn cpu_write(&self, prg_ram: &mut [u8], address: u16, value: u8) {
        match self.mapper {
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
            }
            _ => {
                println!("Invalid address")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prg_nrom_128_mapper_0() {
        let mapper = Mapper::new(0);

        let prg_rom_size = 16 * 1024;
        let mut prg_rom = vec![0xEA; prg_rom_size];
        let prg_ram = vec![0x00; 8 * 1024];

        prg_rom[0] = 0xEB;

        assert_eq!(mapper.cpu_read(&prg_rom, &prg_ram, 0x8000), 0xEB);
        assert_eq!(mapper.cpu_read(&prg_rom, &prg_ram, 0xC000), 0xEB);
    }

    #[test]
    fn prg_nrom_256_mapper_0() {
        let mapper = Mapper::new(0);

        let prg_rom_size = 32 * 1024;
        let mut prg_rom = vec![0xEA; prg_rom_size];
        let prg_ram = vec![0x00; 8 * 1024];

        prg_rom[0] = 0xEB;
        prg_rom[0x4000] = 0xEC;

        assert_eq!(mapper.cpu_read(&prg_rom, &prg_ram, 0x8000), 0xEB);
        assert_eq!(mapper.cpu_read(&prg_rom, &prg_ram, 0xC000), 0xEC);
    }

    #[test]
    fn prg_ram_read_mapper_0() {
        let mapper = Mapper::new(0);

        let prg_rom_size = 16 * 1024;
        let prg_rom = vec![0x00; prg_rom_size];
        let mut prg_ram = vec![0xEA; 8 * 1024];

        prg_ram[0] = 0xEB;
        prg_ram[8191] = 0xEC;

        assert_eq!(mapper.cpu_read(&prg_rom, &prg_ram, 0x6000), 0xEB);
        assert_eq!(mapper.cpu_read(&prg_rom, &prg_ram, 0x7FFF), 0xEC);
    }

   #[test]
    fn prg_ram_write_mapper_0() {
        let mapper = Mapper::new(0);

        let mut prg_ram = vec![0x00; 8 * 1024];

        mapper.cpu_write(&mut prg_ram, 0x6001, 0xFF);

        assert_eq!(prg_ram[1], 0xFF);
    }

    #[test]
    fn prg_ram_write_mirrored_mapper_0() {
        let mapper = Mapper::new(0);

        let mut prg_ram = vec![0x00; 2*1024];

        mapper.cpu_write(&mut prg_ram, 0x6000, 0xFF);
        assert_eq!(prg_ram[0], 0xFF);
        assert_eq!(mapper.cpu_read(&[], &prg_ram, 0x6800), 0xFF);
    }

    #[test]
    fn prg_rom_write_mapper_0() {
        let mapper = Mapper::new(0);

        let mut prg_ram = vec![0x00; 8 * 1024];
        let prg_rom = vec![0xEA; 16 * 1024];

        mapper.cpu_write(&mut prg_ram, 0x8001, 0xFF);

        assert!(prg_ram.iter().all(|&x| x == 0));
        assert!(prg_rom.iter().all(|&x| x == 0xEA));
    }
}
