use std::collections::HashMap;
use crate::cpu::CPU;
use once_cell::sync::Lazy;

//Need an Operation ENUM here
pub enum Operation {
    LDA,
    LDY,
    LDX,
    Other,
}

//AddressMode enum
pub enum AddressMode{
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
    operation: Operation,
    addressing: AddressMode,
    cycles: usize,
}

pub fn handler_dispatch(cpu: &mut CPU, instruction: &Instruction) {
    //Dispatch to correct handler
    //match statement (or something similar) by op
    match instruction.operation {
     Operation::LDA | Operation::LDX | Operation::LDY => CPU::load_memory(cpu, instruction),
     _ => ()
    }
}

pub static OPCODE_LOOKUP: Lazy<HashMap<u8, Instruction>> = Lazy::new(|| {
    let mut m = HashMap::new();
    //LDA Opcodes
    m.insert(0xA9u8, Instruction{operation:Operation::LDA, addressing:AddressMode::Immediate, cycles:2});
    m.insert(0xA5u8, Instruction{operation:Operation::LDA, addressing:AddressMode::ZeroPage, cycles:2});
    m.insert(0xB5u8, Instruction{operation:Operation::LDA, addressing:AddressMode::ZeroPageIndexedX, cycles:2});
    m.insert(0xADu8, Instruction{operation:Operation::LDA, addressing:AddressMode::Absolute, cycles:2});
    m.insert(0xBDu8, Instruction{operation:Operation::LDA, addressing:AddressMode::AbsoluteIndexedX, cycles:2});
    m.insert(0xB9u8, Instruction{operation:Operation::LDA, addressing:AddressMode::AbsoluteIndexedY, cycles:2});
    m.insert(0xA1u8, Instruction{operation:Operation::LDA, addressing:AddressMode::IndexedIndirectX, cycles:2});
    
    //LDX Opcodes
    m.insert(0xB1u8, Instruction{operation:Operation::LDX, addressing:AddressMode::Immediate, cycles:2});
    m.insert(0xA2u8, Instruction{operation:Operation::LDX, addressing:AddressMode::ZeroPage, cycles:2});
    m.insert(0xA6u8, Instruction{operation:Operation::LDX, addressing:AddressMode::ZeroPageIndexedY, cycles:2});
    m.insert(0xAEu8, Instruction{operation:Operation::LDX, addressing:AddressMode::Absolute, cycles:2});
    m.insert(0xBEu8, Instruction{operation:Operation::LDX, addressing:AddressMode::AbsoluteIndexedY, cycles:2});

    //LDY Opcodes
    m.insert(0xA0u8, Instruction{operation:Operation::LDY, addressing:AddressMode::Immediate, cycles:2});
    m.insert(0xA4u8, Instruction{operation:Operation::LDY, addressing:AddressMode::ZeroPage, cycles:2});
    m.insert(0xB4u8, Instruction{operation:Operation::LDY, addressing:AddressMode::ZeroPageIndexedY, cycles:2});
    m.insert(0xACu8, Instruction{operation:Operation::LDY, addressing:AddressMode::Absolute, cycles:2});
    m.insert(0xBCu8, Instruction{operation:Operation::LDY, addressing:AddressMode::AbsoluteIndexedY, cycles:2});
    m
});
