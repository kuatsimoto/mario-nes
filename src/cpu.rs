//Define CPU struct
mod opcode_lookup;
use crate::cpu_bus::CPUBus;
use std::{collections::HashMap, string};

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
    pub fn fetch_byte(&mut self) -> u8 {
        let mem_byte = self.bus_read(self.PC);
        self.PC = self.PC.wrapping_add(1);
        mem_byte
    }
    pub fn load_memory(&mut self, instruction: &opcode_lookup::Instruction) {
        //Will read from memory and load a register with the value

        println!("Function Start!")
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

