//Define CPU struct
mod opcode_lookup;
use crate::{
    cpu::opcode_lookup::{AddressMode, Instruction},
    cpu_bus::CpuBus,
};

enum AddressModeResult {
    Address { address: u16, page_crossed: bool },
    ZeroPage { address: u8 },
    Immediate { value: u8 },
    Accumulator,
    Relative { offset: i8 },
}

#[allow(non_snake_case)]
pub struct CPU<B: CpuBus> {
    //Core registers
    A: u8,
    X: u8,
    Y: u8,
    SP: u8,
    PC: u16,
    P: u8,

    //Internal state
    cycles_remaining: usize,
    halted: bool,

    //Borrow the CPU Bus
    cpu_bus: B,
}
impl<B: CpuBus> CPU<B> {
    pub fn new(cpu_bus: B) -> Self {
        CPU {
            A: 0x00,
            X: 0x00,
            Y: 0x00,
            SP: 0xFD,
            PC: 0x0000,
            P: 0x24,
            cycles_remaining: 0,
            halted: false,
            cpu_bus,
        }
    }
    //Helper functions to set and clear status flags
    pub const CARRY: u8 = 0x01;
    pub const ZERO: u8 = 0x02;
    pub const INTERRUPT: u8 = 0x04;
    pub const DECIMAL: u8 = 0x08;
    pub const BREAK: u8 = 0x10;
    pub const UNUSED: u8 = 0x20;
    pub const OVERFLOW: u8 = 0x40;
    pub const NEGATIVE: u8 = 0x80;

    pub fn set_flag(&mut self, flag: u8, set: bool) {
        self.P = match set {
            true => self.P | flag,
            false => self.P & !flag,
        };
        self.P = self.P | Self::UNUSED;
    }
    pub fn get_flag(&self, flag: u8) -> bool {
        (self.P & flag) != 0
    }
    //Bus helper functions to read and write from the bus
    fn bus_read(&mut self, address: u16) -> u8 {
        self.cpu_bus.cpu_read(address)
    }
    fn bus_write(&mut self, address: u16, value: u8) {
        self.cpu_bus.cpu_write(address, value);
    }
    fn read_u16(&mut self, address: u16) -> u16 {
        //Read from two addresses and return a u16 as a combination of upper and lower bytes
        let upper_byte = self.bus_read(address.wrapping_add(1)) as u16;
        let lower_byte = self.bus_read(address) as u16;

        (upper_byte << 8) | lower_byte
    }
    //Reset function. Resets registers and gets PC from cartridge PRG ROM
    pub fn reset(&mut self) {
        self.A = 0x00;
        self.X = 0x00;
        self.Y = 0x00;
        self.SP = 0xFD;
        self.PC = self.read_u16(0xFFFC);
        self.P = 0x24;
        self.cycles_remaining = 7;
        self.halted = false;
    }

    //Begin Opcode functionality.
    pub fn fetch_pc_byte(&mut self) -> u8 {
        let mem_byte = self.bus_read(self.PC);
        self.PC = self.PC.wrapping_add(1);
        mem_byte
    }
    fn address_mapper(&mut self, address_mode: &AddressMode, operand: &u16) -> AddressModeResult {
        //Need to add +cycles to this for page crossed and branches
        //Probably need some instruction mapper for the implicit mode
        //Relative mode requires a signed operand, cant use the address for this

        fn page_crossed(address: &u16, new_address: u16) -> bool {
            //Check if page was crossed.
            //Compare MSB from address and new_address. If MSB is changed, page was crossed

            let msb_address = address & 0xFF00;
            let msb_new_address = new_address & 0xFF00;

            msb_address != msb_new_address
        }

        // let mut result = AddressModeResult {
        //     value: 0,
        //     address: 0x0000,
        //     rel_offset: 0,
        //     page_crossed: false,
        //     accumulator: false,
        // };

        match address_mode {
            AddressMode::ZeroPageIndexedX => {
                let address = (*operand as u8).wrapping_add(self.X);
                AddressModeResult::ZeroPage { address: address }
            }
            AddressMode::ZeroPageIndexedY => {
                let address = (*operand as u8).wrapping_add(self.Y);
                AddressModeResult::ZeroPage { address: address }
            }
            AddressMode::AbsoluteIndexedX => {
                let address = operand + self.X as u16;
                AddressModeResult::Address {
                    address: address,
                    page_crossed: page_crossed(operand, address),
                }
            }
            AddressMode::AbsoluteIndexedY => {
                let address = operand + self.Y as u16;
                AddressModeResult::Address {
                    address: address,
                    page_crossed: page_crossed(operand, address),
                }
            }
            AddressMode::IndexedIndirectX => {
                let address = self.bus_read(((*operand as u8).wrapping_add(self.X)) as u16) as u16
                    + self.bus_read((*operand as u8).wrapping_add(self.X).wrapping_add(1) as u16)
                        as u16
                        * 256;
                AddressModeResult::Address {
                    address: address,
                    page_crossed: false,
                }
            }
            AddressMode::IndexedIndirectY => {
                let base_address = self.bus_read((*operand as u8) as u16) as u16
                    + (self.bus_read((*operand as u8).wrapping_add(1) as u16) as u16 * 256);
                let address = base_address + self.Y as u16;

                AddressModeResult::Address {
                    address: address,
                    page_crossed: address & 0xFF00 != base_address & 0xFF00,
                }
            }
            AddressMode::Accumulator => AddressModeResult::Accumulator,
            AddressMode::Immediate => AddressModeResult::Immediate {
                value: *operand as u8,
            },
            AddressMode::ZeroPage => AddressModeResult::ZeroPage {
                address: *operand as u8,
            },
            AddressMode::Absolute => AddressModeResult::Address {
                address: *operand,
                page_crossed: false,
            },
            // AddressMode::Relative => self.bus_read(self.PC + address), //Need to figure this out
            // AddressMode::Indirect => //Need to figure this out,
            _ => AddressModeResult::Immediate { value: 0x00 }, //Will be changed later for correct error
                                                               //handling
        }
    }
    pub fn load_memory(
        &mut self,
        instruction: &Instruction,
        operand: u16,
    ) -> Result<(), &'static str> {
        //Will read from memory and load a register with the value
        let address_mode_result = match self.address_mapper(&instruction.addressing, &operand) {
            AddressModeResult::Address {
                address,
                page_crossed,
            } => {
                let value = self.bus_read(address);
                Ok((value, page_crossed))
            }
            AddressModeResult::ZeroPage { address } => {
                let value = self.bus_read(address as u16);
                Ok((value, false))
            }
            AddressModeResult::Immediate { value } => Ok((value, false)),
            _ => Err("Invalid Address Mode"),
        };

        let (value, page_crossed) = address_mode_result?;

        let cycles = if page_crossed {instruction.cycles + 1} else {instruction.cycles};
        self.cycles_remaining = cycles;

        self.set_flag(Self::NEGATIVE, value & 0x80 != 0);
        self.set_flag(Self::ZERO, value == 0);

        match instruction.operation {
            opcode_lookup::Operation::LDA => self.A = value,
            opcode_lookup::Operation::LDY => self.Y = value,
            opcode_lookup::Operation::LDX => self.X = value,
            _ => return Err("Load instruction failed"),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //Things to test
    //Registers and reset func
    //Flag Helpers
    //Address mapper
    //Load memory handler
    //Bus read/write

    use std::result;

    use crate::cpu::opcode_lookup::Operation;

    use super::*;

    struct MockBus {
        mem: [u8; 65536],
        reads: Vec<u16>,
        writes: Vec<u16>,
    }
    impl CpuBus for MockBus {
        fn cpu_read(&mut self, address: u16) -> u8 {
            self.reads.push(address);
            self.mem[address as usize]
        }
        fn cpu_write(&mut self, address: u16, value: u8) {
            self.writes.push(address);
            self.mem[address as usize] = value;
        }
    }
    impl MockBus {
        fn new() -> Self {
            MockBus {
                mem: [0u8; 65536],
                reads: vec![],
                writes: vec![],
            }
        }
    }

    #[test]
    fn test_reset() {
        let mut bus = MockBus::new();
        bus.mem[0xFFFC] = 0xEA;
        bus.mem[0xFFFD] = 0xEB;

        let mut cpu = CPU::new(bus);

        cpu.set_flag(CPU::<MockBus>::CARRY, true);
        cpu.set_flag(CPU::<MockBus>::INTERRUPT, false);
        cpu.A = 0xFF;
        cpu.X = 0xEA;
        cpu.halted = true;
        cpu.cycles_remaining = 3;

        cpu.reset();

        assert_eq!(cpu.A, 0x00);
        assert_eq!(cpu.X, 0x00);
        assert_eq!(cpu.Y, 0x00);
        assert_eq!(cpu.PC, 0xEBEA);
        assert_eq!(cpu.SP, 0xFD);
        assert_eq!(cpu.P, 0x24);
        assert_eq!(cpu.halted, false);
        assert_eq!(cpu.cycles_remaining, 7);
    }

    #[test]
    fn test_flag_helpers(){
        let bus = MockBus::new();
        let mut cpu = CPU::new(bus);

        cpu.set_flag(CPU::<MockBus>::CARRY, true);
        cpu.set_flag(CPU::<MockBus>::DECIMAL, true);
        cpu.set_flag(CPU::<MockBus>::INTERRUPT, false);

        assert_ne!(cpu.P & CPU::<MockBus>::CARRY, 0);
        assert_ne!(cpu.P & CPU::<MockBus>::DECIMAL, 0);
        assert_eq!(cpu.P & CPU::<MockBus>::INTERRUPT, 0);
    }

    #[test]
    fn test_address_mapper(){
        let mut bus = MockBus::new();
        bus.mem[0x0001] = 0xEA;
        bus.mem[0x0002] = 0xEB;
        bus.mem[0x0003] = 0xEC;
        bus.mem[0x00FF] = 0xFF;
        
        let mut cpu = CPU::new(bus);

        let result = cpu.address_mapper(&AddressMode::Absolute, &0x1234);
        match result {
            AddressModeResult::Address{address, page_crossed} => {
                assert_eq!(address, 0x1234);
                assert!(!page_crossed);
            }
            _=> panic!("Absolute mode failed")
        };
        
        cpu.X = 0x01;
        let result = cpu.address_mapper(&AddressMode::ZeroPageIndexedX, &0x00FF);
        match result {
            AddressModeResult::ZeroPage{address} => {
                assert_eq!(address, 0x0000);
            }
            _=> panic!("Zero Page X mode failed")
        };
       
        cpu.X = 0;
        cpu.Y = 0x01;
        let result = cpu.address_mapper(&AddressMode::ZeroPageIndexedY, &0x00FF);
        match result {
            AddressModeResult::ZeroPage{address} => {
                assert_eq!(address, 0x0000);
            }
            _=> panic!("Zero Page Y mode failed")
        };

        cpu.X = 0;
        cpu.Y = 0;
        let result = cpu.address_mapper(&AddressMode::ZeroPage, &0x00FF);
        match result {
            AddressModeResult::ZeroPage{address} => {
                assert_eq!(address, 0x00FF);
            }
            _=> panic!("Zero Page mode failed")
        };

        cpu.X = 2;
        cpu.Y = 0;
        let result = cpu.address_mapper(&AddressMode::AbsoluteIndexedX, &0x00FF);
        match result {
            AddressModeResult::Address{address, page_crossed} => {
                assert_eq!(address, 0x0101);
                assert_eq!(page_crossed, true);
            }
            _=> panic!("Absoulte Indexed X mode failed")
        };
        
        cpu.X = 0;
        cpu.Y = 2;
        let result = cpu.address_mapper(&AddressMode::AbsoluteIndexedY, &0x00FF);
        match result {
            AddressModeResult::Address{address, page_crossed} => {
                assert_eq!(address, 0x0101);
                assert_eq!(page_crossed, true);
            }
            _=> panic!("Absolute Indexed Y mode failed")
        };

        cpu.X = 2;
        cpu.Y = 0;
        let result = cpu.address_mapper(&AddressMode::IndexedIndirectX, &0x00FF);
        match result {
            AddressModeResult::Address{address, page_crossed} => {
                assert_eq!(address, 0xEBEA);
                assert_eq!(page_crossed, false);
            }
            _=> panic!("Indexed Indirect X mode failed")
        };

        cpu.X = 0;
        cpu.Y = 2;
        let result = cpu.address_mapper(&AddressMode::IndexedIndirectY, &0x00FF);
        match result {
            AddressModeResult::Address{address, page_crossed} => {
                assert_eq!(cpu.Y, 2);
                assert_eq!(address, 0x0101);
                assert_eq!(page_crossed, true);
            }
            _=> panic!("Indexed Indirect Y mode failed")
        };

        cpu.X = 0;
        cpu.Y = 0;
        let result = cpu.address_mapper(&AddressMode::Accumulator, &0x0001);
        match result {
            AddressModeResult::Accumulator => {()},
            _ => panic!("Accumulator mode failed"),
        };

        cpu.X = 0;
        cpu.Y = 0;
        let result = cpu.address_mapper(&AddressMode::Immediate, &0x0001);
        match result {
            AddressModeResult::Immediate { value } => {
                assert_eq!(value, 0x0001);
            },
            _ => panic!("Immediate mode failed"),
        };
    }

    #[test]
    fn test_load_handler() {
        let mut bus = MockBus::new();
        bus.mem[0x0002] = 0xEA;
        bus.mem[0x0100] = 0xEB;
        let mut cpu = CPU::new(bus);

        //Immediate, LDA
        let opcode = 0xA9u8;
        let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).expect("Invalid instruction");
        cpu.load_memory(instruction, 0x0001).expect("Failed to load memory");
        assert_eq!(cpu.A, 0x0001);
        assert_eq!(cpu.cycles_remaining, 2);

        //ZeroPageIndexedY, LDX
        cpu.Y = 0x01;
        cpu.A = 0x00;
        let opcode = 0xB6u8;
        let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).expect("Invalid Instruction");
        cpu.load_memory(instruction, 0x0001).expect("Failed to load memory");
        assert_eq!(cpu.X, 0x00EA);
        assert_eq!(cpu.cycles_remaining, 4);

        //AbsoluteIndexedY, LDY
        cpu.Y = 0x01;
        let opcode = 0xBCu8;
        let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).expect("Invalid Instruction");
        cpu.load_memory(instruction, 0x00FF).expect("Failed to load memory");
        assert_eq!(cpu.Y, 0xEB);address & 0xFF00 != base_address & 0xFF00
        assert_eq!(cpu.cycles_remaining, 5);
    }
}
