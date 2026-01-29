#[cfg(test)]
mod tests;
mod opcode_lookup;
use std::ops::Add;

use crate::{
    cpu::opcode_lookup::{AddressMode, Instruction, Operation},
    cpu_bus::CpuBus,
};

enum WrapMode {
    JmpIndirect,
    Normal,
}

enum AddressModeResult {
    Address { address: u16, page_crossed: bool },
    ZeroPage { address: u8 },
    Immediate { value: u8 },
    Accumulator,
    Relative { offset: i8 },
    Indirect { pointer: u16 },
    Implicit,
    Default,
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
    fn read_u16(&mut self, address: u16, wrap_mode: WrapMode) -> u16 {
        //Read from two addresses and return a u16 as a combination of upper and lower bytes
        let lower_byte = self.bus_read(address) as u16;
        let upper_byte = match wrap_mode {
            WrapMode::JmpIndirect => {
               if address & 0x00FF == 0x00FF {
                   self.bus_read(address & 0xFF00) as u16
               } else {
                   self.bus_read(address.wrapping_add(1)) as u16
               }
            },
            _ => self.bus_read(address.wrapping_add(1)) as u16,
        };

        (upper_byte << 8) | lower_byte
    }
    fn push_to_stack(&mut self, value: u8) {
        let stack_address = self.SP as u16 + 0x0100;
        self.bus_write(stack_address, value);
        self.SP = self.SP.wrapping_sub(1);
    }
    fn pull_from_stack(&mut self) -> u8 {
        self.SP = self.SP.wrapping_add(1);
        let stack_address = self.SP as u16 + 0x0100;
        let value = self.bus_read(stack_address);
        value
    }
    //Reset function. Resets registers and gets PC from cartridge PRG ROM
    pub fn reset(&mut self) {
        self.A = 0x00;
        self.X = 0x00;
        self.Y = 0x00;
        self.SP = 0xFD;
        self.PC = self.read_u16(0xFFFC, WrapMode::Normal);
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
            AddressMode::Relative => AddressModeResult::Relative{offset: *operand as i8}, 
            AddressMode::Indirect => AddressModeResult::Indirect{pointer: *operand},
            AddressMode::Implicit => AddressModeResult::Implicit,
            // _ => AddressModeResult::Immediate { value: 0x00 }, //Will be changed later for correct error
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

        let cycles = if page_crossed {
            instruction.cycles + 1
        } else {
            instruction.cycles
        };
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

    pub fn store_memory(&mut self, instruction: &Instruction, operand: u16) -> Result<(), &'static str> {
        let address_mode_result = match self.address_mapper(&instruction.addressing, &operand) {
            AddressModeResult::ZeroPage { address } => {
                Ok(address as u16)
            }
            AddressModeResult::Address { address, .. } => {
                Ok(address)
            }
            _ => Err("Invalid address mode")
        };

        let address = match address_mode_result {
            Ok(a) => a,
            Err(e) => return Err(e)
        };

        match instruction.operation {
            opcode_lookup::Operation::STA => self.bus_write(address, self.A),
            opcode_lookup::Operation::STX => self.bus_write(address, self.X),
            opcode_lookup::Operation::STY => self.bus_write(address, self.Y),
            _ => return Err("Invalid operation")
        };

        Ok(())
    }



    pub fn arithmetic_operation(
        &mut self,
        instruction: &Instruction,
        operand: u16,
    ) -> Result<(), &'static str> {
        //Read value from memory or value
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

        let (value, page_crossed) = match address_mode_result {
            Ok((v, p)) => (v, p),
            Err(e) => return Err(e),
        };

        self.cycles_remaining = if page_crossed {
            instruction.cycles + 1
        } else {
            instruction.cycles
        };

        let mut add_with_carry = | value: u8, add: bool| {
            let prev_reg_a = self.A;
            let carry = if self.get_flag(Self::CARRY) {1} else {0};
            let result = self.A as u16 + value as u16 + carry as u16; //Calc as u16 for carry
            self.A = result as u8;

            match add {
                true => self.set_flag(Self::CARRY, result > 0xFF),
                false => self.set_flag(Self::CARRY, result as u8 & 0x80 != 0),
            }
            //All flag logic is the same for subtraction
            self.set_flag(Self::ZERO, result as u8 == 0);
            //If first bit of value is different than 1st bit of result AND
            //1st bit of A is different than 1st bit of result, overflow occurs
            self.set_flag(Self::OVERFLOW, ((value ^ result as u8) & (prev_reg_a ^ result as u8)) & 0x80 !=0);
            self.set_flag(Self::NEGATIVE, result as u8 & 0x80 != 0);
        };

        match instruction.operation {
            opcode_lookup::Operation::ADC => {
                add_with_carry(value, true);
            },
            opcode_lookup::Operation::SBC => {
                add_with_carry(!value, true);
            } 
            _ => return Err("Invalid operation")
        };
        Ok(())
    }

    pub fn bitwise_logic(& mut self, instruction: &Instruction, operand: u16) -> Result<(), &'static str> {
        //Read value from memory or value
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

        let (value, page_crossed) = match address_mode_result {
            Ok((v, p)) => (v, p),
            Err(e) => return Err(e),
        };

        self.cycles_remaining = if page_crossed {
            instruction.cycles + 1
        } else {
            instruction.cycles
        };

        match instruction.operation {
            opcode_lookup::Operation::AND => {
                let result = self.A & value;
                self.set_flag(CPU::<B>::ZERO, result == 0);
                self.set_flag(CPU::<B>::NEGATIVE, result & 0x80 != 0);
                self.A = result;
            },
            opcode_lookup::Operation::EOR => {
                let result = self.A ^ value;
                self.set_flag(CPU::<B>::ZERO, result == 0);
                self.set_flag(CPU::<B>::NEGATIVE, result & 0x80 != 0);
                self.A = result
            },
            opcode_lookup::Operation::ORA => {
                let result = self.A | value;
                self.set_flag(CPU::<B>::ZERO, result == 0);
                self.set_flag(CPU::<B>::NEGATIVE, result & 0x80 != 0);
                self.A = result
            },
            _ => return Err("Invalid operation")
        };

        Ok(())

    }

    pub fn set_flag_operation(&mut self, instruction: &Instruction) -> Result<(), &'static str> {
        match instruction.addressing {
            AddressMode::Implicit => (),
            _ => return Err("Invalid address mode")
        }
        match instruction.operation {
            Operation::CLC => self.set_flag(CPU::<B>::CARRY, false),
            Operation::CLD => self.set_flag(CPU::<B>::DECIMAL, false),
            Operation::CLI => self.set_flag(CPU::<B>::INTERRUPT, false),
            Operation::CLV => self.set_flag(CPU::<B>::OVERFLOW, false),
            Operation::SEC => self.set_flag(CPU::<B>::CARRY, true),
            Operation::SEI => self.set_flag(CPU::<B>::INTERRUPT, true),
            Operation::SED => self.set_flag(CPU::<B>::DECIMAL, true),
            _ => return Err("Invalid operation")
        };

        Ok(())
    }
    fn take_branch(&mut self, offset: i8, instruction: &Instruction) {
        let new_pc = (self.PC as i16 + offset as i16) as u16;
        self.cycles_remaining = if new_pc & 0xFF00 != self.PC & 0xFF00 {
            instruction.cycles + 2
        } else {
            instruction.cycles + 1
        };

        self.PC = new_pc; 
    }

    pub fn branch_operation(&mut self, instruction: &Instruction, operand: u16) -> Result<(), &'static str> {
        let address_mode_result = match self.address_mapper(&instruction.addressing, &operand) {
            AddressModeResult::Relative{offset} => Ok(offset),
            _ => Err("Invalid address mode")
        };        
        
        let offset = match address_mode_result {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        self.cycles_remaining = instruction.cycles;
        
        match instruction.operation {
            Operation::BCC => {
                if !self.get_flag(CPU::<B>::CARRY) {
                    self.take_branch(offset, &instruction)
                }
            },
            Operation::BCS => {
                if self.get_flag(CPU::<B>::CARRY){
                    self.take_branch(offset, &instruction);
                }
            }
            Operation::BEQ => {
                if self.get_flag(CPU::<B>::ZERO){
                    self.take_branch(offset, &instruction);
                }
            }
            Operation::BMI => {
                if self.get_flag(CPU::<B>::NEGATIVE){
                    self.take_branch(offset, &instruction);
                }
            }
            Operation::BNE => {
                if !self.get_flag(CPU::<B>::ZERO){
                    self.take_branch(offset, &instruction);
                }
            }
            Operation::BPL => {
                if !self.get_flag(CPU::<B>::NEGATIVE){
                    self.take_branch(offset, &instruction);
                }
            }
            Operation::BVC => {
                if !self.get_flag(CPU::<B>::OVERFLOW){
                    self.take_branch(offset, &instruction);
                }
            }
            Operation::BVS => {
                if self.get_flag(CPU::<B>::OVERFLOW){
                    self.take_branch(offset, &instruction);
                }
            }
            _ => return Err("Invalid operation")
        };

        Ok(())
    }

    fn set_tranfer_flags(&mut self, value: u8){
        self.set_flag(CPU::<B>::ZERO, value == 0);
        self.set_flag(CPU::<B>::NEGATIVE, value & 0x80 != 0);
    } 
    pub fn transfer_operations(&mut self, instruction: &Instruction) -> Result<(), &'static str>{
        match instruction.addressing {
            AddressMode::Implicit => (),
            _ => return Err("Invalid address mode")
        };
        
        self.cycles_remaining = instruction.cycles;

        match instruction.operation {
            Operation::TAX =>  {
                let value = self.A;
                self.X = value;
                self.set_tranfer_flags(value);
            },
            Operation::TAY => {
                let value = self.A;
                self.Y = value;
                self.set_tranfer_flags(value);
            },
            Operation::TSX => {
                let value = self.SP;
                self.X = value;
                self.set_tranfer_flags(value);
            },
            Operation::TXA => {
                let value = self.X;
                self.A = value;
                self.set_tranfer_flags(value);
            },
            Operation::TXS => self.SP = self.X,
            Operation::TYA => {
                let value = self.Y;
                self.A = value;
                self.set_tranfer_flags(value);
            },
            _ => return Err("Invalid operation"),
        };

        Ok(())

    }
    
    pub fn jump_operations(&mut self, instruction: &Instruction, operand: u16) -> Result<(), &'static str>{
        
        let address_mode_result = match self.address_mapper(&instruction.addressing, &operand) {
            AddressModeResult::Address{address, ..} => {
                Ok(address)
            }, 
            AddressModeResult::Indirect{pointer} => {
                let address = self.read_u16(pointer, WrapMode::JmpIndirect);
                Ok(address)
            },
            AddressModeResult::Implicit => {
                Ok(0)
            }
            _ => Err("Invalid address mode")
        };

        let address = match address_mode_result {
            Ok(v) => v,
            Err(e) => return Err(e)
        };

        self.cycles_remaining = instruction.cycles;

        match instruction.operation {
            Operation::JMP => {
                self.PC = address;
            }
            Operation::JSR => {
                self.push_to_stack(((self.PC.wrapping_sub(1) & 0xFF00) >> 8) as u8);
                self.push_to_stack(self.PC.wrapping_sub(1) as u8);
                self.PC = address;
           },
           Operation::RTS => {
               let pc_low = self.pull_from_stack();
               let pc_high = self.pull_from_stack();
               self.PC = ((pc_high as u16) << 8 | pc_low as u16).wrapping_add(1);
           },
           Operation::BRK => {
                self.push_to_stack(((self.PC & 0xFF00) >> 8) as u8);
                self.push_to_stack(self.PC as u8);

                self.push_to_stack(self.P | CPU::<B>::BREAK | CPU::<B>::UNUSED); //Push Break
                                                                                 //without setting
                                                                                 //flag
                self.set_flag(CPU::<B>::INTERRUPT, true);
                self.PC = self.read_u16(0xFFFE, WrapMode::Normal)
           }
            Operation::RTI => {
                let flags = self.pull_from_stack();
                self.P = (flags | CPU::<B>::UNUSED) & !CPU::<B>::BREAK; //Need to set Unused and
                                                                        //clear Break
                let pc_low = self.pull_from_stack();
                let pc_high = self.pull_from_stack();

                self.PC = ((pc_high as u16) << 8) | pc_low as u16;
            }
            _ => return Err("Invalid operation")
        }

        Ok(())
    }

    pub fn shift_operations(&mut self, instruction: &Instruction, operand: u16) -> Result<(), &'static str> {
        let address_mode_result = match self.address_mapper(&instruction.addressing, &operand) {
            AddressModeResult::Accumulator => {
                let address = 0;
                let accumulator = true;

                Ok((address, accumulator))
            },
            AddressModeResult::ZeroPage { address } => {
                let address = address as u16;
                let accumulator = false;

                Ok((address, accumulator))
            },
            AddressModeResult::Address { address, .. } => {
                let address = address as u16;
                let accumulator = false;

                Ok((address, accumulator))
            },
            _ => {
                Err("Invalid Address mode")
            }
        };

        let (address, accumulator) = match address_mode_result {
            Ok((v,a)) => (v,a),
            Err(e) => return Err(e)
        };
        
        self.cycles_remaining = instruction.cycles;

        match instruction.operation {
            Operation::ASL => {
                let value = match accumulator {
                    true => self.A,
                    false => self.bus_read(address)
                };
                
                let result = (value as u16) << 1;
                self.set_flag(CPU::<B>::CARRY, result & 0x100 != 0);
                self.set_flag(CPU::<B>::ZERO, result & 0x00FF == 0);
                self.set_flag(CPU::<B>::NEGATIVE, result & 0x80 != 0);

                match accumulator {
                    true => self.A = result as u8,
                    false => self.bus_write(address, result as u8),
                }
            }
            Operation::LSR => {
                let value = match accumulator {
                    true => self.A,
                    false => self.bus_read(address),
                };

                self.set_flag(CPU::<B>::CARRY, value & 0x01 != 0);
                let result = value >> 1;

                self.set_flag(CPU::<B>::ZERO, result == 0);
                self.set_flag(CPU::<B>::NEGATIVE, false);

                match accumulator {
                    true => self.A = result,
                    false => self.bus_write(address, result),
                }
            }
            Operation::ROL => {
                let value = match accumulator {
                    true => self.A,
                    false => self.bus_read(address),
                };

                let carry_in = self.get_flag(CPU::<B>::CARRY);

                let result = value << 1 | carry_in as u8;
                self.set_flag(CPU::<B>::CARRY, value & 0x80 != 0);
                self.set_flag(CPU::<B>::ZERO, result == 0);
                self.set_flag(CPU::<B>::NEGATIVE, result & 0x80 != 0);

                match accumulator {
                    true => self.A = result,
                    false => self.bus_write(address, result),
                };
            }
            Operation::ROR => {
                let value = match accumulator {
                    true => self.A,
                    false => self.bus_read(address),
                };

                let carry_in = self.get_flag(CPU::<B>::CARRY);

                let result = value >> 1 | (carry_in as u8) << 7;
                self.set_flag(CPU::<B>::CARRY, value & 0x01 != 0);
                self.set_flag(CPU::<B>::ZERO, result == 0);
                self.set_flag(CPU::<B>::NEGATIVE, result & 0x80 != 0);

                match accumulator {
                    true => self.A = result,
                    false => self.bus_write(address, result),
                }
            }
            _ => {
                return Err("Invalid operation")
            }
        }

        Ok(())
    }

    pub fn compare_operations(&mut self, instruction: &Instruction, operand: u16) -> Result<(), &'static str> {
        let address_mode_result = match self.address_mapper(&instruction.addressing, &operand) {
            AddressModeResult::Address { address, page_crossed } => {
                let value = self.bus_read(address);
                Ok((value, page_crossed))
            },
            AddressModeResult::ZeroPage { address } => {
                let value = self.bus_read(address as u16);
                Ok((value, false))
            },
            AddressModeResult::Immediate { value } => {
                Ok((value, false))
            }
            _ => Err("Invalid address mode")
        };

        let (value, page_crossed) = match address_mode_result {
            Ok((v,p)) => (v,p),
            Err(e) => return Err(e),
        };

        self.cycles_remaining = match page_crossed {
            true => instruction.cycles + 1,
            false => instruction.cycles,
        };

        let register = match instruction.operation {
            Operation::CMP => self.A,
            Operation::CPX => self.X,
            Operation::CPY => self.Y,
            _ => return Err("Invalid operation")
        };

        let result = register.wrapping_sub(value);
        self.set_flag(CPU::<B>::CARRY, register >= value);
        self.set_flag(CPU::<B>::ZERO, register == value);
        self.set_flag(CPU::<B>::NEGATIVE, result & 0x80 !=0);

        Ok(())
    }

    pub fn stack_operations(&mut self, instruction: &Instruction) -> Result<(), &'static str> {
        match instruction.addressing {
            AddressMode::Implicit => (),
            _ => return Err("Invalid address mode")
        };

        self.cycles_remaining = instruction.cycles;

        match instruction.operation {
            Operation::PHA => self.push_to_stack(self.A),
            Operation::PHP => self.push_to_stack(self.P | CPU::<B>::UNUSED | CPU::<B>::BREAK),
            Operation::PLA => {
                let value = self.pull_from_stack();
                self.A = value;
                self.set_flag(CPU::<B>::ZERO, value == 0);
                self.set_flag(CPU::<B>::NEGATIVE, value & 0x80 !=0);
            },
            Operation::PLP => self.P = (self.pull_from_stack() | CPU::<B>::UNUSED) & !CPU::<B>::BREAK,
            _ => return Err("Invalid operation")
        }

        Ok(())
    } 
    
    pub fn nop_operation(&mut self, instruction: &Instruction) -> Result<(), &'static str> {
        match instruction.addressing {
            AddressMode::Implicit => (),
            _ => return Err("Invalid address mode")
        };

        self.cycles_remaining = instruction.cycles;
        match instruction.operation {
            Operation::NOP => (),
            _ => return Err("Invalid operation")
        };
        Ok(())
    }
}

