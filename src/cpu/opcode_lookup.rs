use crate::cpu::CPU;
use once_cell::sync::Lazy;
use std::collections::HashMap;

//Need an Operation ENUM here
pub enum Operation {
    LDA,
    LDY,
    LDX,
    Other,
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

pub fn handler_dispatch(cpu: &mut CPU, instruction: &mut Instruction, operand: u16) -> Result<(), &'static str> {
    //Dispatch to correct handler
    //match statement (or something similar) by op
    match instruction.operation {
        Operation::LDA | Operation::LDX | Operation::LDY => CPU::load_memory(cpu, instruction, operand),
        _ => return Err("Invalid Operation"),
    };

    Ok(())
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

    m
});
