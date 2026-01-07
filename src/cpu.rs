//Define CPU struct
mod opcode_lookup;
use crate::{
    cpu::opcode_lookup::{AddressMode, Instruction},
    cpu_bus::CPUBus,
};

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
    fn address_mapper(
        &mut self,
        address_mode: AddressMode,
        address: &u16,
        instruction: &mut Instruction,
    ) -> u8 {
        //Need to add +cycles to this for page crossed and branches
        //Probably need some instruction mapper for the implicit mode
        //Relative mode requires a signed operand, cant use the address for this
        
        fn page_crossed (address: &u16, new_address: u16) -> bool {
            //Check if page was crossed.
            //Compare MSB from address and new_address. If MSB is changed, page was crossed
            
            let msb_address = address & 0xFF00;
            let msb_new_address = new_address & 0xFF00;

            msb_address != msb_new_address

        }

        match address_mode {
            AddressMode::ZeroPageIndexedX => self.bus_read((address + self.X as u16) % 256),
            AddressMode::ZeroPageIndexedY => self.bus_read((address + self.Y as u16) % 256),
            AddressMode::AbsoluteIndexedX => {
                let new_address = address + self.X as u16;
                match page_crossed(address, new_address){
                    true => instruction.cycles = 5,
                    false => instruction.cycles = 4
                };
                self.bus_read(new_address)
            },
            AddressMode::AbsoluteIndexedY => {
                let new_address = address + self.Y as u16;
                match page_crossed(address, new_address){
                    true => instruction.cycles = 5,
                    false => instruction.cycles = 4
                };
                self.bus_read(new_address)
            },
            AddressMode::IndexedIndirectX => self.bus_read(
                (address + self.X as u16) % 256 + ((address + self.X as u16 + 1) % 256) * 256
            ),
            AddressMode::IndexedIndirectY => {
                let new_address = ((address + 1) % 256) * 256 + self.Y as u16;
                match page_crossed(address, new_address){
                    true => instruction.cycles = 5,
                    false => instruction.cycles = 4
                };
                self.bus_read(new_address)
            },
            AddressMode::Accumulator => self.A,
            AddressMode::Immediate => *address as u8,
            AddressMode::ZeroPage => self.bus_read(address & 0x00FF),
            AddressMode::Absolute => self.bus_read(*address),
            // AddressMode::Relative => self.bus_read(self.PC + address), //Need to figure this out
            // AddressMode::Indirect => //Need to figure this out,
            // AddressMode::Implicit => 0x0000, //Need to figure this one out
            _ => 0,
        }
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
