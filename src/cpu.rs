//Define CPU struct
mod opcode_lookup;
use crate::{
    cpu::opcode_lookup::{AddressMode, Instruction},
    cpu_bus::CPUBus,
};

// struct AddressModeResult {
//     value: u8,
//     address: u16,
//     rel_offset: i8,
//     page_crossed: bool,
//     accumulator: bool,
// }

enum AddressModeResult {
    Address { address: u16, page_crossed: bool },
    ZeroPage { address: u8 },
    Immediate { value: u8 },
    Accumulator,
    Relative { offset: i8 },
}

#[allow(non_snake_case)]
pub struct CPU<'a> {
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
    cpu_bus: &'a mut CPUBus,
}
impl<'a> CPU<'a> {
    pub fn new(cpu_bus: &'a mut CPUBus) -> Self {
        CPU {
            A: 0x00,
            X: 0x00,
            Y: 0x00,
            SP: 0x00,
            PC: 0xFFFC,
            P: 0x20,
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
        self.P = 0x34;
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
                //Needs page_crossed here.
                AddressModeResult::Address {
                    address: address,
                    page_crossed: address < base_address,
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
        instruction: &mut Instruction,
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

        if page_crossed {
            instruction.cycles = instruction.cycles + 1
        };
        self.cycles_remaining = instruction.cycles;

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

// fn test_func(x: u8) {
//     println!("{x}")
// }
// fn test_hash() {
//     type Operation = Box<dyn Fn(u8)>;
//
//     let mut opcode_lookup: HashMap<u8, (u8,Operation)> = HashMap::new();
//     opcode_lookup.insert(0xA0u8, (0x01, Box::new(test_func)));
//
//     let operation = opcode_lookup.get(&0xA0u8);
//     operation.unwrap().1(operation.unwrap().0)
// }
