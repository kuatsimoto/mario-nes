use crate::{cpu::{CPU,CpuBus}, cpu_bus::NesBus};
use once_cell::sync::Lazy;
use std::collections::HashMap;

//Need an Operation ENUM here
pub enum Operation {
    LDA,
    LDY,
    STA,
    STX,
    STY,
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
    SEC,
    SED,
    SEI,
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    JMP,
    JSR,
    RTS,
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
        Operation::CLC | Operation::CLD | Operation::CLI | Operation::CLV | Operation::SEC | Operation::SED | Operation::SEI  => {
            let result = CPU::bitwise_logic(cpu, instruction, operand);
            match result {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        }
        Operation::STA | Operation::STX | Operation::STY => {
            let result = CPU::store_memory(cpu, instruction, operand);
            match result {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        }
        Operation::BCC | Operation::BCS | Operation::BEQ | Operation::BNE | Operation::BPL | Operation::BVC | Operation::BVS => {
            let result = CPU::branch_operation(cpu, instruction, operand);
            match result {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        }
        Operation::TAX | Operation::TAY | Operation::TXA | Operation::TYA | Operation::TSX | Operation::TXS => {
            let result = CPU::transfer_operations(cpu, instruction);
            match result {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            }
        }
        _ => Err("Invalid")
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
        0x85u8,
        Instruction {
            operation: Operation::STA,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        }
    );
    m.insert(
        0x95u8,
        Instruction {
            operation: Operation::STA,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x8Du8,
        Instruction {
            operation: Operation::STA,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
    );
    m.insert(
        0x9Du8,
        Instruction {
            operation: Operation::STA,
            addressing: AddressMode::AbsoluteIndexedX,
            cycles: 5,
        }
    );
    m.insert(
        0x99u8,
        Instruction {
            operation: Operation::STA,
            addressing: AddressMode::AbsoluteIndexedY,
            cycles: 5,
        }
    );
    m.insert(
        0x81u8,
        Instruction {
            operation: Operation::STA,
            addressing: AddressMode::IndexedIndirectX,
            cycles: 6,
        }
    );
    m.insert(
        0x91u8,
        Instruction {
            operation: Operation::STA,
            addressing: AddressMode::IndexedIndirectY,
            cycles: 2,
        }
    );
    m.insert(
        0x86u8,
        Instruction {
            operation: Operation::STX,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        }
    );
    m.insert(
        0x96u8,
        Instruction {
            operation: Operation::STX,
            addressing: AddressMode::ZeroPageIndexedY,
            cycles: 4,
        }
    );
    m.insert(
        0x8Eu8,
        Instruction {
            operation: Operation::STX,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
    );
    m.insert(
        0x84u8,
        Instruction {
            operation: Operation::STY,
            addressing: AddressMode::ZeroPage,
            cycles: 3,
        }
    );
    m.insert(
        0x94u8,
        Instruction {
            operation: Operation::STY,
            addressing: AddressMode::ZeroPageIndexedX,
            cycles: 4,
        }
    );
    m.insert(
        0x8Cu8,
        Instruction {
            operation: Operation::STY,
            addressing: AddressMode::Absolute,
            cycles: 4,
        }
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
            operation: Operation::CLV,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x38u8,
        Instruction{
            operation: Operation::SEC,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0xF8u8,
        Instruction{
            operation: Operation::SED,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x78u8,
        Instruction{
            operation: Operation::SEI,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x90u8,
        Instruction{
            operation: Operation::BCC,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0xB0u8,
        Instruction{
            operation: Operation::BCS,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0xF0u8,
        Instruction{
            operation: Operation::BEQ,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0x30u8,
        Instruction{
            operation: Operation::BMI,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0xD0u8,
        Instruction{
            operation: Operation::BNE,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0x10u8,
        Instruction{
            operation: Operation::BPL,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0x50u8,
        Instruction{
            operation: Operation::BVC,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0x70u8,
        Instruction{
            operation: Operation::BVS,
            addressing: AddressMode::Relative,
            cycles: 2,
        }
    );
    m.insert(
        0xAAu8,
        Instruction{
            operation: Operation::TAX,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0xA8u8,
        Instruction{
            operation: Operation::TAY,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0xBAu8,
        Instruction{
            operation: Operation::TSX,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x8Au8,
        Instruction{
            operation: Operation::TXA,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x9Au8,
        Instruction{
            operation: Operation::TXS,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x98u8,
        Instruction{
            operation: Operation::TYA,
            addressing: AddressMode::Implicit,
            cycles: 2,
        }
    );
    m.insert(
        0x4Cu8,
        Instruction{
            operation: Operation::JMP,
            addressing: AddressMode::Absolute,
            cycles: 3,
        }
    );
    m.insert(
        0x6Cu8,
        Instruction{
            operation: Operation::JMP,
            addressing: AddressMode::Indirect,
            cycles: 5,
        }
    );
    m.insert(
        0x20u8,
        Instruction{
            operation: Operation::JSR,
            addressing: AddressMode::Absolute,
            cycles: 6,
        }
    );
    m.insert(
        0x69u8,
        Instruction{
            operation: Operation::RTS,
            addressing: AddressMode::Implicit,
            cycles: 6,
        }
    );



    m
});
