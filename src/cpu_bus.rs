pub struct CPUBus {
    mapper: super::mapper::Mapper,
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
}
impl CPUBus {
   pub fn cpu_read(&self, address: u16) -> u8 {
        //Routes reads to appropriate memory component (Mapper, CPU mem)
        //Currently, only routes to mapper, all other addresses are ignored

        match address {
            0x6000..=0xFFFF => self.mapper.cpu_read(&self.prg_rom, &self.prg_ram, address),
            _ => 0,
        }
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        //Routes writes to appropriate memory component (Mapper, CPU mem)
        //Currently only routes to mapper, all other addresses are ignored

        match address {
            0x6000..=0xFFFF => self.mapper.cpu_write(&mut self.prg_ram, address, value),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper::Mapper;

    fn new_test_bus(prg_rom_size: usize, mapper_number: usize) -> CPUBus {
        let prg_rom = vec![0xEA; prg_rom_size];
        let prg_ram = vec![0x00; 8*1024];

        let mapper = Mapper::new(mapper_number);

        CPUBus {
            mapper,
            prg_rom,
            prg_ram,
        }
    }

    #[test]
    fn bus_rom_read() {
        //create CPU Bus with 16KiB of ROM
        let cpu_bus = new_test_bus(16 * 1024, 0);

        assert_eq!(cpu_bus.cpu_read(0x8000), 0xEA);
    }
    
    #[test]
    fn bad_bus_rom_read() {
        //Address 0x0000 is still not mapped. Will be mapped to CPU RAM later. Should return 0
        let cpu_bus = new_test_bus(16*1024, 0);

        assert_eq!(cpu_bus.cpu_read(0x0000), 0);
    }
    
    #[test]
    fn bus_ram_write() {
        let mut cpu_bus = new_test_bus(16*1024, 0);

        cpu_bus.cpu_write(0x6000, 0x67);
        assert_eq!(cpu_bus.cpu_read(0x6000), 0x67);
    }

    #[test]
    fn bad_bus_ram_write() {
        let mut cpu_bus = new_test_bus(16*1024, 0);

        cpu_bus.cpu_write(0x0000, 0x67);
        
        //Check all bytes of RAM are still 0 after bad write
        assert!(cpu_bus.prg_ram.iter().all(|&x| x == 0))
    }
}
