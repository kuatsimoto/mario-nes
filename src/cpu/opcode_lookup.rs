use crate::{cpu::{CPU,CpuBus}, cpu_bus::NesBus};
use once_cell::sync::Lazy;
use std::collections::HashMap;

//Need an Operation ENUM here
pub enum Operation {
    LDA,
    LDY,
    LDX,
    ADC,
    SBC,
    AND,
    EOR,
    ORA,
    CLC,
    CLD,
    CLI,
    CLV,
}

//AddressMode enum
pub enum AddressMode {
    ZeroPageIndexedX,
    ZeroPageIndexedY,
    AbsoluteIndexedX,
    AbsoluteIndexedY,
    IndexedIndirectX,
    IndexedIndirectY,
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    Absolute,
    Relative,
    Indirect,
}

pub struct Instruction {
    pub operation: Operation,
    pub addressing: AddressMode,
    pub cycles: usize,
}

pub fn handler_dispatch(cpu: &mut CPU<NesBus>, instruction: &mut Instruction, operand: u16) -> Result<(), &'static str> {
    //Dispatch to correct handler
    //match statement (or something similar) by op
    match instruction.operation {
        Operation::LDA | Operation::LDX | Operation::LDY => {
            let result = CPU::load_memory(cpu, instruction, operand);
            match result{
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        },
        Operation::ADC | Operation::SBC => { 
            let result = CPU::arithmetic_operation(cpu, instruction, operand);
            match result{
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        },
        Operation::AND | Operation::EOR | Operation::ORA => {
            let result = CPU::bitwise_logic(cpu, instruction, operand);
            match result {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        }
        Operation::CLC | Operation::CLD | Operation::CLI | Operation::CLV => {
            let result = CPU::bitwise_logic(cpu, instruction, operand);
            match result {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        }
        // _ => Err("Invalid operation in handler dispatch"),
    }
}

//NOTE: Cycles need to be either an enum or a hashmap.
//Need to detect cases where it can be higher if page crossed

pub static OPCODE_LOOKUP: Lazy<HashMap<u8, Instruction>> = Lazy::new(|| {
    let mut m = HashMap::new();
    //LDA Opcodes
    m.insert(
        0xA9u8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::Immediate,
            cycles: 2,
        },
    );
    m.insert(
        0xA5u8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        },
    );
    m.insert(
        0xB5u8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        },
    );
    m.insert(
        0xADu8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::Absolute,
            cycles: 4,
        },
    );
    m.insert(
        0xBDu8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::AbsoluteIndexedX,
            cycles: 4,
        },
    );
    m.insert(
        0xB9u8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        },
    );
    m.insert(
        0xA1u8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::IndexedIndirectX,
            cycles: 6,
        },
    );
    m.insert(
        0xB1u8,
        Instruction {
            operation: Operation::LDA,
            addressing: AddressMode::IndexedIndirectY,
            cycles: 5,
        },
    );

    //LDX Opcodes
    m.insert(
        0xA2u8,
        Instruction {
            operation: Operation::LDX,
            addressing: AddressMode::Immediate,
            cycles: 2,
        },
    );
    m.insert(
        0xA6u8,
        Instruction {
            operation: Operation::LDX,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        },
    );
    m.insert(
        0xB6u8,
        Instruction {
            operation: Operation::LDX,
            addressing: AddressMode::ZeroPageIndexedY,
            cycles: 4,
        },
    );
    m.insert(
        0xAEu8,
        Instruction {
            operation: Operation::LDX,
            addressing: AddressMode::Absolute,
            cycles: 4,
        },
    );
    m.insert(
        0xBEu8,
        Instruction {
            operation: Operation::LDX,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        },
    );

    //LDY Opcodes
    m.insert(
        0xA0u8,
        Instruction {
            operation: Operation::LDY,
            addressing: AddressMode::Immediate,
            cycles: 2,
        },
    );
    m.insert(
        0xA4u8,
        Instruction {
            operation: Operation::LDY,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        },
    );
    m.insert(
        0xB4u8,
        Instruction {
            operation: Operation::LDY,
            addressing: AddressMode::ZeroPageIndexedY,
            cycles: 4,
        },
    );
    m.insert(
        0xACu8,
        Instruction {
            operation: Operation::LDY,
            addressing: AddressMode::Absolute,
            cycles: 4,
        },
    );
    m.insert(
        0xBCu8,
        Instruction {
            operation: Operation::LDY,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        },
    );
    m.insert(
        0x69u8,//NICE
        Instruction {
            operation: Operation::ADC,
            addressing: AddressMode::Immediate,
            cycles: 2,
        }
    );
    m.insert(
        0x65u8,
        Instruction{
            operation: Operation::ADC,
            addressing: AddressMode::ZeroPage,
            cycles: 2,
        }
    );
    m.insert(
        0x75u8,
        Instruction{
            operation: Operation::ADC,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x6Du8,
        Instruction{
            operation: Operation::ADC,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
    );
    m.insert(
        0x7Du8,
        Instruction{
            operation: Operation::ADC,
            addressing: AddressMode::AbsoluteIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x79u8,
        Instruction{
            operation: Operation::ADC,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        }
    );
    m.insert(
        0x61u8,
        Instruction{
            operation: Operation::ADC,
            addressing: AddressMode::IndexedIndirectX,
            cycles: 6,
        }
    );
    m.insert(
        0x71u8,
        Instruction{
            operation: Operation::ADC,
            addressing: AddressMode::IndexedIndirectY,
            cycles: 5,
        }
    );
    m.insert(
        0xE9u8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::Immediate,
            cycles: 2,
        }
    );
    m.insert(
        0xE5u8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        }
    );
    m.insert(
        0xF5u8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0xEDu8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
    );
    m.insert(
        0xFDu8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::AbsoluteIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0xF9u8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        }
    );
    m.insert(
        0xE1u8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::IndexedIndirectX,
            cycles: 6,
        }
    );
    m.insert(
        0xF1u8,
        Instruction{
            operation: Operation::SBC,
            addressing: AddressMode::IndexedIndirectY,
            cycles: 5,
        }
    );
    m.insert(
        0x29u8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::Immediate,
            cycles: 2,
        }
    );
    m.insert(
        0x25u8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        }
    );
    m.insert(
        0x35u8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x2Du8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
    );
    m.insert(
        0x3Du8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::AbsoluteIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x39u8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        }
    );
    m.insert(
        0x21u8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::IndexedIndirectX,
            cycles: 6,
        }
    );
    m.insert(
        0x31u8,
        Instruction{
            operation: Operation::AND,
            addressing: AddressMode::IndexedIndirectY,
            cycles: 5,
        }
    );
    m.insert(
        0x49u8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::Immediate,
            cycles: 2,
        }
    );
    m.insert(
        0x45u8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        }
    );
    m.insert(
        0x55u8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x4Du8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
    );
    m.insert(
        0x5Du8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::AbsoluteIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x59u8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        }
    );
    m.insert(
        0x41u8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::IndexedIndirectX,
            cycles: 6,
        }
    );
    m.insert(
        0x51u8,
        Instruction{
            operation: Operation::EOR,
            addressing: AddressMode::IndexedIndirectY,
            cycles: 5,
        }
    );
    m.insert(
        0x09u8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::Immediate,
            cycles: 2,
        }
    );
    m.insert(
        0x05u8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        }
    );
    m.insert(
        0x15u8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x0Du8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
    );
    m.insert(
        0x1Du8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::AbsoluteIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x19u8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 4,
        }
    );
    m.insert(
        0x01u8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::IndexedIndirectX,
            cycles: 6,
        }
    );
    m.insert(
        0x11u8,
        Instruction{
            operation: Operation::ORA,
            addressing: AddressMode::IndexedIndirectY,
            cycles: 5,
        }
    );
    m.insert(
        0x18u8,
        Instruction{
            operation: Operation::CLC,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0xD8u8,
        Instruction{
            operation: Operation::CLD,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x58u8,
        Instruction{
            operation: Operation::CLI,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0xB8u8,
        Instruction{
            operation: Operation::CLI,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );


    m
});
