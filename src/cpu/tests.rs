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

//function to clear all flags
fn clear_all_flags(cpu: &mut CPU<MockBus>) {
    cpu.P = 0x20;
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
fn test_flag_helpers() {
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
fn test_address_mapper() {
    let mut bus = MockBus::new();
    bus.mem[0x0001] = 0xEA;
    bus.mem[0x0002] = 0xEB;
    bus.mem[0x0003] = 0xEC;
    bus.mem[0x00FF] = 0xFF;

    let mut cpu = CPU::new(bus);

    let result = cpu.address_mapper(&AddressMode::Absolute, &0x1234);
    match result {
        AddressModeResult::Address {
            address,
            page_crossed,
        } => {
            assert_eq!(address, 0x1234);
            assert!(!page_crossed);
        }
        _ => panic!("Absolute mode failed"),
    };

    cpu.X = 0x01;
    let result = cpu.address_mapper(&AddressMode::ZeroPageIndexedX, &0x00FF);
    match result {
        AddressModeResult::ZeroPage { address } => {
            assert_eq!(address, 0x0000);
        }
        _ => panic!("Zero Page X mode failed"),
    };

    cpu.X = 0;
    cpu.Y = 0x01;
    let result = cpu.address_mapper(&AddressMode::ZeroPageIndexedY, &0x00FF);
    match result {
        AddressModeResult::ZeroPage { address } => {
            assert_eq!(address, 0x0000);
        }
        _ => panic!("Zero Page Y mode failed"),
    };

    cpu.X = 0;
    cpu.Y = 0;
    let result = cpu.address_mapper(&AddressMode::ZeroPage, &0x00FF);
    match result {
        AddressModeResult::ZeroPage { address } => {
            assert_eq!(address, 0x00FF);
            let result = cpu.address_mapper(&AddressMode::Absolute, &0x1234);
            match result {
                AddressModeResult::Address {
                    address,
                    page_crossed,
                } => {
                    assert_eq!(address, 0x1234);
                    assert!(!page_crossed);
                }
                _ => panic!("Absolute mode failed"),
            };
        }
        _ => panic!("Zero Page mode failed"),
    };

    cpu.X = 2;
    cpu.Y = 0;
    let result = cpu.address_mapper(&AddressMode::AbsoluteIndexedX, &0x00FF);
    match result {
        AddressModeResult::Address {
            address,
            page_crossed,
        } => {
            assert_eq!(address, 0x0101);
            assert_eq!(page_crossed, true);
        }
        _ => panic!("Absoulte Indexed X mode failed"),
    };

    cpu.X = 0;
    cpu.Y = 2;
    let result = cpu.address_mapper(&AddressMode::AbsoluteIndexedY, &0x00FF);
    match result {
        AddressModeResult::Address {
            address,
            page_crossed,
        } => {
            assert_eq!(address, 0x0101);
            assert_eq!(page_crossed, true);
        }
        _ => panic!("Absolute Indexed Y mode failed"),
    };

    cpu.X = 2;
    cpu.Y = 0;
    let result = cpu.address_mapper(&AddressMode::IndexedIndirectX, &0x00FF);
    match result {
        AddressModeResult::Address {
            address,
            page_crossed,
        } => {
            assert_eq!(address, 0xEBEA);
            assert_eq!(page_crossed, false);
        }
        _ => panic!("Indexed Indirect X mode failed"),
    };

    cpu.X = 0;
    cpu.Y = 2;
    let result = cpu.address_mapper(&AddressMode::IndexedIndirectY, &0x00FF);
    match result {
        AddressModeResult::Address {
            address,
            page_crossed,
        } => {
            assert_eq!(cpu.Y, 2);
            assert_eq!(address, 0x0101);
            assert_eq!(page_crossed, true);
        }
        _ => panic!("Indexed Indirect Y mode failed"),
    };

    cpu.X = 0;
    cpu.Y = 0;
    let result = cpu.address_mapper(&AddressMode::Accumulator, &0x0001);
    match result {
        AddressModeResult::Accumulator => (),
        _ => panic!("Accumulator mode failed"),
    };

    cpu.X = 0;
    cpu.Y = 0;
    let result = cpu.address_mapper(&AddressMode::Immediate, &0x0001);
    match result {
        AddressModeResult::Immediate { value } => {
            assert_eq!(value, 0x0001);
        }
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
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.load_memory(instruction, 0x0001)
        .expect("Failed to load memory");
    assert_eq!(cpu.A, 0x0001);
    assert_eq!(cpu.cycles_remaining, 2);

    //ZeroPageIndexedY, LDX
    cpu.Y = 0x01;
    cpu.A = 0x00;
    let opcode = 0xB6u8;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid Instruction");
    cpu.load_memory(instruction, 0x0001)
        .expect("Failed to load memory");
    assert_eq!(cpu.X, 0x00EA);
    assert_eq!(cpu.cycles_remaining, 4);

    //AbsoluteIndexedY, LDY
    cpu.Y = 0x01;
    let opcode = 0xBCu8;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid Instruction");
    cpu.load_memory(instruction, 0x00FF)
        .expect("Failed to load memory");
    assert_eq!(cpu.Y, 0xEB);
    assert_eq!(cpu.cycles_remaining, 5);
}

#[test]
fn test_arithmetic_operation() {
    let mut bus = MockBus::new();
    bus.mem[0x0001] = 0x01;
    bus.mem[0x0100] = 0x80;
    bus.mem[0x0002] = 0xFF;
    bus.mem[0x0003] = 0x00;
    bus.mem[0x0004] = 0x03;

    let mut cpu = CPU::new(bus);

    cpu.set_flag(CPU::<MockBus>::CARRY, false);
    cpu.A = 0xFF;
    let opcode = 0x69u8; //NICE
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.arithmetic_operation(instruction, 0x02)
        .expect("ADC operation failed");
    assert_eq!(cpu.A, 0x01);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert_eq!(cpu.cycles_remaining, 2);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::OVERFLOW));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test ADC with Zero Flag
    clear_all_flags(&mut cpu);
    cpu.A = 0xFF;
    cpu.X = 0x01;
    let opcode = 0x75u8;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.arithmetic_operation(instruction, 0x00)
        .expect("ADC operation failed");
    assert_eq!(cpu.A, 0x00);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert_eq!(cpu.cycles_remaining, 4);
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::OVERFLOW));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test ADC overflow flag
    clear_all_flags(&mut cpu);
    cpu.A = 0x80;
    cpu.Y = 0x01;
    let opcode = 0x79u8; //NICE
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.arithmetic_operation(instruction, 0xFF)
        .expect("ADC operation failed");
    assert_eq!(cpu.A, 0x0);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert_eq!(cpu.cycles_remaining, 5);
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(cpu.get_flag(CPU::<MockBus>::OVERFLOW));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test SBC Immediate
    clear_all_flags(&mut cpu);
    cpu.A = 0x04;
    cpu.set_flag(CPU::<MockBus>::CARRY, true);
    let opcode = 0xE9u8;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.arithmetic_operation(instruction, 0x02)
        .expect("SBC operation failed");
    assert_eq!(cpu.A, 0x02);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert_eq!(cpu.cycles_remaining, 2);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::OVERFLOW));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test SBC IndexedIndirectY
    clear_all_flags(&mut cpu);
    cpu.A = 0x04;
    cpu.Y = 0x01;
    cpu.set_flag(CPU::<MockBus>::CARRY, true);
    let opcode = 0xF1u8;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.arithmetic_operation(instruction, 0x02)
        .expect("SBC operation failed");
    assert_eq!(cpu.A, 0x84);
    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY));
    assert_eq!(cpu.cycles_remaining, 6);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(cpu.get_flag(CPU::<MockBus>::OVERFLOW));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test SBC AbsoluteIndexedX
    clear_all_flags(&mut cpu);
    cpu.A = 0x04;
    cpu.X = 0x01;
    cpu.set_flag(CPU::<MockBus>::CARRY, false);
    let opcode = 0xFDu8;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.arithmetic_operation(instruction, 0x03)
        .expect("SBC operation failed");
    assert_eq!(cpu.A, 0x00);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert_eq!(cpu.cycles_remaining, 4);
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::OVERFLOW));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_bitwise_logic() {
    let mut bus = MockBus::new();
    bus.mem[0x0001] = 0x80;
    bus.mem[0x0002] = 0x02;

    let mut cpu = CPU::new(bus);

    //Test AND immediate
    clear_all_flags(&mut cpu);
    cpu.A = 0x03u8;
    let opcode = 0x29;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x01)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x01);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test AND ZeroPageIndexedX;
    clear_all_flags(&mut cpu);
    cpu.A = 0x83u8;
    cpu.X = 0x01;
    let opcode = 0x35;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x00)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x80);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test AND Absoulte;
    clear_all_flags(&mut cpu);
    cpu.A = 0x80u8;
    let opcode = 0x2D;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x02)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x00);
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test EOR immediate
    clear_all_flags(&mut cpu);
    cpu.A = 0x03u8;
    let opcode = 0x49;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x01)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x02);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test EOR ZeroPageIndexedX;
    clear_all_flags(&mut cpu);
    cpu.A = 0x03u8;
    cpu.X = 0x01;
    let opcode = 0x55;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x00)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x83);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test EOR Absoulte;
    clear_all_flags(&mut cpu);
    cpu.A = 0x02u8;
    let opcode = 0x4D;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x02)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x00);
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test ORA immediate
    clear_all_flags(&mut cpu);
    cpu.A = 0x03u8;
    let opcode = 0x09;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x01)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x03);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test ORA ZeroPageIndexedX;
    clear_all_flags(&mut cpu);
    cpu.A = 0x03u8;
    cpu.X = 0x01;
    let opcode = 0x15;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x00)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x83);
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));

    //Test ORA Absoulte;
    clear_all_flags(&mut cpu);
    cpu.A = 0x00u8;
    let opcode = 0x4D;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.bitwise_logic(instruction, 0x03)
        .expect("EOR operation failed");
    assert_eq!(cpu.A, 0x00);
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_clear_flag_operations() {
    //Clear carry
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    clear_all_flags(&mut cpu);
    cpu.set_flag(CPU::<MockBus>::CARRY, true);
    let opcode = 0x18u8;
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");
    cpu.set_flag_operation(instruction)
        .expect("CLC operatio failed");
    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY))
}

#[test]
fn test_bne_not_taken() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    cpu.PC = 0x1000;
    cpu.set_flag(CPU::<MockBus>::ZERO, true); // BNE should NOT branch

    let opcode = 0xD0u8; // BNE
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");

    cpu.branch_operation(instruction, 0x05)
        .expect("Branch failed");

    // PC unchanged
    assert_eq!(cpu.PC, 0x1000);
    // Base cycles only
    assert_eq!(cpu.cycles_remaining, instruction.cycles);
}

#[test]
fn test_bne_taken_no_page_cross() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    cpu.PC = 0x1000;
    cpu.set_flag(CPU::<MockBus>::ZERO, false); // BNE should branch

    let opcode = 0xD0u8; // BNE
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");

    // +5 offset
    cpu.branch_operation(instruction, 0x05)
        .expect("Branch failed");

    assert_eq!(cpu.PC, 0x1005);
    assert_eq!(cpu.cycles_remaining, instruction.cycles + 1);
}

#[test]
fn test_bne_taken_page_cross() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    cpu.PC = 0x10FE;
    cpu.set_flag(CPU::<MockBus>::ZERO, false); // branch taken

    let opcode = 0xD0u8; // BNE
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");

    // +2  crosses from 0x10 to 0x11
    cpu.branch_operation(instruction, 0x02)
        .expect("Branch failed");

    assert_eq!(cpu.PC, 0x1100);
    assert_eq!(cpu.cycles_remaining, instruction.cycles + 2);
}

#[test]
fn test_bmi_negative_offset() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    cpu.PC = 0x2000;
    cpu.set_flag(CPU::<MockBus>::NEGATIVE, true); // BMI taken

    let opcode = 0x30u8; // BMI
    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&opcode)
        .expect("Invalid instruction");

    // -2 offset (0xFE as i8)
    cpu.branch_operation(instruction, 0xFE)
        .expect("Branch failed");

    assert_eq!(cpu.PC, 0x1FFE);
    assert_eq!(cpu.cycles_remaining, instruction.cycles + 2);
}

#[test]
fn test_bcs_vs_bcc() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    cpu.PC = 0x3000;
    cpu.set_flag(CPU::<MockBus>::CARRY, true);

    // BCC should NOT branch
    let bcc = opcode_lookup::OPCODE_LOOKUP.get(&0x90).unwrap();
    cpu.branch_operation(bcc, 0x10).unwrap();
    assert_eq!(cpu.PC, 0x3000);

    // BCS SHOULD branch
    let bcs = opcode_lookup::OPCODE_LOOKUP.get(&0xB0).unwrap();
    cpu.branch_operation(bcs, 0x10).unwrap();
    assert_eq!(cpu.PC, 0x3010);
}

#[test]
fn test_tax_transfer_flags() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    // A -> X (non-zero, non-negative)
    cpu.A = 0x01;
    let opcode = 0xAAu8; // TAX
    let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).unwrap();

    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.X, 0x01);
    assert_eq!(cpu.P & CPU::<MockBus>::ZERO, 0);
    assert_eq!(cpu.P & CPU::<MockBus>::NEGATIVE, 0);
    assert_eq!(cpu.cycles_remaining, instruction.cycles);

    // A -> X (zero)
    cpu.A = 0x00;
    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.X, 0x00);
    assert_ne!(cpu.P & CPU::<MockBus>::ZERO, 0);

    // A -> X (negative)
    cpu.A = 0x80;
    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.X, 0x80);
    assert_ne!(cpu.P & CPU::<MockBus>::NEGATIVE, 0);
}

#[test]
fn test_tay_and_tya() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    // TAY
    cpu.A = 0xFF;
    let opcode = 0xA8u8; // TAY
    let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).unwrap();

    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.Y, 0xFF);
    assert_ne!(cpu.P & CPU::<MockBus>::NEGATIVE, 0);

    // TYA
    let opcode = 0x98u8; // TYA
    let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).unwrap();

    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.A, 0xFF);
    assert_ne!(cpu.P & CPU::<MockBus>::NEGATIVE, 0);
}

#[test]
fn test_tsx_sets_flags() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    cpu.SP = 0x00;
    let opcode = 0xBAu8; // TSX
    let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).unwrap();

    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.X, 0x00);
    assert_ne!(cpu.P & CPU::<MockBus>::ZERO, 0);

    cpu.SP = 0x80;
    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.X, 0x80);
    assert_ne!(cpu.P & CPU::<MockBus>::NEGATIVE, 0);
}

#[test]
fn test_txs_does_not_touch_flags() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    // Set flags beforehand
    cpu.set_flag(CPU::<MockBus>::ZERO, true);
    cpu.set_flag(CPU::<MockBus>::NEGATIVE, true);

    cpu.X = 0x12;

    let opcode = 0x9Au8; // TXS
    let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).unwrap();

    cpu.transfer_operations(instruction).unwrap();

    assert_eq!(cpu.SP, 0x12);
    assert_ne!(cpu.P & CPU::<MockBus>::ZERO, 0);
    assert_ne!(cpu.P & CPU::<MockBus>::NEGATIVE, 0);
}

#[test]
fn test_transfer_invalid_address_mode() {
    let bus = MockBus::new();
    let mut cpu = CPU::new(bus);

    let instruction = Instruction {
        operation: Operation::TAX,
        addressing: AddressMode::Immediate,
        cycles: 2,
        // fill in any other required fields with sane defaults
    };

    let result = cpu.transfer_operations(&instruction);
    assert!(result.is_err());
}

#[test]
fn test_jmp_absolute() {
    let mut cpu = CPU::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x4C) // JMP abs
        .unwrap();

    cpu.PC = 0x2000;

    cpu.jump_operations(instruction, 0x1234)
        .expect("JMP absolute failed");

    assert_eq!(cpu.PC, 0x1234);
    assert_eq!(cpu.cycles_remaining, instruction.cycles);
}

#[test]
fn test_jmp_indirect() {
    let mut bus = MockBus::new();

    // Pointer = 0x30FF  wraps to 0x3000
    bus.mem[0x30FF] = 0xCD;
    bus.mem[0x3000] = 0xAB;

    let mut cpu = CPU::new(bus);

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x6C) // JMP indirect
        .unwrap();

    cpu.jump_operations(instruction, 0x30FF)
        .expect("JMP indirect failed");

    assert_eq!(cpu.PC, 0xABCD);
    assert_eq!(cpu.cycles_remaining, instruction.cycles);
}

#[test]
fn test_jsr() {
    let mut cpu = CPU::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x20) // JSR
        .unwrap();

    cpu.PC = 0x4000;
    cpu.SP = 0xFF;

    cpu.jump_operations(instruction, 0x1234)
        .expect("JSR failed");

    // PC - 1 pushed
    assert_eq!(cpu.bus_read(0x01FF), 0x3F);
    assert_eq!(cpu.bus_read(0x01FE), 0xFF);

    assert_eq!(cpu.SP, 0xFD);
    assert_eq!(cpu.PC, 0x1234);
}

#[test]
fn test_rts() {
    let mut cpu = CPU::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x60) // RTS
        .unwrap();

    cpu.SP = 0xFD;
    cpu.bus_write(0x01FE, 0x34);
    cpu.bus_write(0x01FF, 0x12);

    cpu.jump_operations(instruction, 0)
        .expect("RTS failed");

    assert_eq!(cpu.PC, 0x1235);
    assert_eq!(cpu.SP, 0xFF);
}

#[test]
fn test_brk() {
    let mut bus = MockBus::new();
    bus.mem[0xFFFE] = 0x78;
    bus.mem[0xFFFF] = 0x56;

    let mut cpu = CPU::new(bus);

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x00) // BRK
        .unwrap();

    cpu.PC = 0x3000;
    cpu.SP = 0xFF;
    cpu.P = 0x00;

    cpu.jump_operations(instruction, 0)
        .expect("BRK failed");

    assert_eq!(cpu.bus_read(0x01FF), 0x30);
    assert_eq!(cpu.bus_read(0x01FE), 0x00);

    let flags = cpu.bus_read(0x01FD);
    assert!(flags & CPU::<MockBus>::BREAK != 0);
    assert!(flags & CPU::<MockBus>::UNUSED != 0);

    assert!(cpu.get_flag(CPU::<MockBus>::INTERRUPT));
    assert_eq!(cpu.PC, 0x5678);
}

#[test]
fn test_rti() {
    let mut cpu = CPU::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x40) // RTI
        .unwrap();

    cpu.SP = 0xFC;
    cpu.bus_write(0x01FD, CPU::<MockBus>::BREAK | CPU::<MockBus>::NEGATIVE);
    cpu.bus_write(0x01FE, 0x34);
    cpu.bus_write(0x01FF, 0x12);

    cpu.jump_operations(instruction, 0)
        .expect("RTI failed");

    assert_eq!(cpu.PC, 0x1234);
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));
    assert!(!cpu.get_flag(CPU::<MockBus>::BREAK));
    assert!(cpu.get_flag(CPU::<MockBus>::UNUSED));
}

#[test]
fn test_jump_invalid_address_mode() {
    let mut cpu = CPU::new(MockBus::new());

    let instruction = Instruction {
        operation: Operation::JMP,
        addressing: AddressMode::Immediate,
        cycles: 3,
    };

    let result = cpu.jump_operations(&instruction, 0);
    assert!(result.is_err());
}


#[test]
fn test_asl_accumulator() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x0A) // ASL A
        .unwrap();

    cpu.A = 0b0100_0001;

    cpu.shift_operations(instruction, 0)
        .expect("ASL accumulator failed");

    assert_eq!(cpu.A, 0b1000_0010);
    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
}

#[test]
fn test_asl_zeropage() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x06) // ASL zp
        .unwrap();

    cpu.bus_write(0x0042, 0x80);

    cpu.shift_operations(instruction, 0x0042)
        .expect("ASL zeropage failed");

    let result = cpu.bus_read(0x0042);
    assert_eq!(result, 0);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_lsr_accumulator() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x4A) // LSR A
        .unwrap();

    cpu.A = 0b0000_0001;

    cpu.shift_operations(instruction, 0)
        .expect("LSR accumulator failed");

    assert_eq!(cpu.A, 0);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_lsr_absolute() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x4E) // LSR abs
        .unwrap();

    cpu.bus_write(0x1234, 0b0000_0010);

    cpu.shift_operations(instruction, 0x1234)
        .expect("LSR absolute failed");

    let result = cpu.bus_read(0x1234);
    assert_eq!(result, 0b0000_0001);
    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_rol_accumulator_with_carry() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x2A) // ROL A
        .unwrap();

    cpu.A = 0b0111_1111;
    cpu.set_flag(CPU::<MockBus>::CARRY, true);

    cpu.shift_operations(instruction, 0)
        .expect("ROL accumulator failed");

    assert_eq!(cpu.A, 0b1111_1111);
    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
}

#[test]
fn test_rol_zeropage_sets_carry() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x26) // ROL zp
        .unwrap();

    cpu.bus_write(0x0040, 0b1000_0000);
    cpu.set_flag(CPU::<MockBus>::CARRY, false);

    cpu.shift_operations(instruction, 0x40)
        .expect("ROL zeropage failed");

    let result = cpu.bus_read(0x0040);
    assert_eq!(result, 0);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
}

#[test]
fn test_ror_accumulator_with_carry() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x6A) // ROR A
        .unwrap();

    cpu.A = 0b0000_0000;
    cpu.set_flag(CPU::<MockBus>::CARRY, true);

    cpu.shift_operations(instruction, 0)
        .expect("ROR accumulator failed");

    assert_eq!(cpu.A, 0b1000_0000);
    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_ror_absolute_sets_carry() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x6E) // ROR abs
        .unwrap();

    cpu.bus_write(0x2000, 0b0000_0001);
    cpu.set_flag(CPU::<MockBus>::CARRY, false);

    cpu.shift_operations(instruction, 0x2000)
        .expect("ROR absolute failed");

    let result = cpu.bus_read(0x2000);
    assert_eq!(result, 0);
    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
}

#[test]
fn test_shift_invalid_address_mode() {
    let mut cpu = CPU::<MockBus>::new(MockBus::new());

    let instruction = Instruction {
        operation: Operation::ASL,
        addressing: AddressMode::Immediate,
        cycles: 2,
    };

    let result = cpu.shift_operations(&instruction, 0x10);
    assert!(result.is_err());
}

#[test]
fn test_cmp_immediate_greater() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    cpu.A = 0x50;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xC9) // CMP immediate
        .unwrap();

    cpu.compare_operations(instruction, 0x0040).unwrap();

    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_cmp_immediate_equal() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    cpu.A = 0x42;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xC9) // CMP immediate
        .unwrap();

    cpu.compare_operations(instruction, 0x0042).unwrap();

    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_cmp_immediate_less() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    cpu.A = 0x10;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xC9) // CMP immediate
        .unwrap();

    cpu.compare_operations(instruction, 0x0020).unwrap();

    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_cpx_zeropage() {
    let bus = MockBus::new();

    let mut cpu = CPU::<MockBus>::new(bus);
    cpu.bus_write(0x0020, 0x10);

    cpu.X = 0x10;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xE4) // CPX zeropage
        .unwrap();

    cpu.compare_operations(instruction, 0x0020).unwrap();

    assert!(cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_cpy_zeropage_negative() {
    let bus = MockBus::new();

    let mut cpu = CPU::<MockBus>::new(bus);
    cpu.bus_write(0x0030, 0x80);
    cpu.Y = 0x40;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xC4) // CPY zeropage
        .unwrap();

    cpu.compare_operations(instruction, 0x0030).unwrap();

    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY));
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_cmp_absolute_x_page_cross() {
    let bus = MockBus::new();

    let mut cpu = CPU::<MockBus>::new(bus);
    cpu.bus_write(0x0100, 0x20);

    cpu.A = 0x30;
    cpu.X = 0x01;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xDD) // CMP absolute,X
        .unwrap();

    cpu.compare_operations(instruction, 0x00FF).unwrap();

    assert_eq!(cpu.cycles_remaining, instruction.cycles + 1);
}

#[test]
fn test_compare_invalid_address_mode() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    let instruction = Instruction {
        operation: Operation::CMP,
        addressing: AddressMode::Accumulator,
        cycles: 2,
    };

    let result = cpu.compare_operations(&instruction, 0);

    assert!(result.is_err());
}

#[test]
fn test_pha_pushes_accumulator() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    cpu.A = 0x42;
    cpu.SP = 0xFD;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x48) // PHA
        .unwrap();

    cpu.stack_operations(instruction).unwrap();

    assert_eq!(cpu.SP, 0xFC);
    assert_eq!(cpu.bus_read(0x01FD), 0x42);
}

#[test]
fn test_php_pushes_status_with_break_and_unused_set() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    cpu.P = 0b0010_0001; // random flags
    cpu.SP = 0xFF;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x08) // PHP
        .unwrap();

    cpu.stack_operations(instruction).unwrap();

    let pushed = cpu.bus_read(0x01FF);

    assert_eq!(cpu.SP, 0xFE);
    assert!(pushed & CPU::<MockBus>::BREAK != 0);
    assert!(pushed & CPU::<MockBus>::UNUSED != 0);
}

#[test]
fn test_pla_sets_zero_flag() {
    let bus = MockBus::new();

    let mut cpu = CPU::<MockBus>::new(bus);
    cpu.bus_write(0x01FE, 0x00);

    cpu.SP = 0xFD;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x68) // PLA
        .unwrap();

    cpu.stack_operations(instruction).unwrap();

    assert_eq!(cpu.A, 0x00);
    assert_eq!(cpu.SP, 0xFE);
    assert!(cpu.get_flag(CPU::<MockBus>::ZERO));
    assert!(!cpu.get_flag(CPU::<MockBus>::NEGATIVE));
}

#[test]
fn test_pla_sets_negative_flag() {
    let bus = MockBus::new();

    let mut cpu = CPU::<MockBus>::new(bus);
    cpu.bus_write(0x01FE, 0x80);

    cpu.SP = 0xFD;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x68) // PLA
        .unwrap();

    cpu.stack_operations(instruction).unwrap();

    assert_eq!(cpu.A, 0x80);
    assert!(cpu.get_flag(CPU::<MockBus>::NEGATIVE));
    assert!(!cpu.get_flag(CPU::<MockBus>::ZERO));
}

#[test]
fn test_plp_restores_flags_correctly() {
    let bus = MockBus::new();

    let mut cpu = CPU::<MockBus>::new(bus);
    cpu.bus_write(0x01FF, 0b1111_1111);

    cpu.SP = 0xFE;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x28) // PLP
        .unwrap();

    cpu.stack_operations(instruction).unwrap();

    assert_eq!(cpu.SP, 0xFF);
    assert!(cpu.get_flag(CPU::<MockBus>::UNUSED));
    assert!(!cpu.get_flag(CPU::<MockBus>::BREAK));
}

#[test]
fn test_stack_operation_sets_cycles() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0x48) // PHA
        .unwrap();

    cpu.stack_operations(instruction).unwrap();

    assert_eq!(cpu.cycles_remaining, instruction.cycles);
}

#[test]
fn test_stack_invalid_address_mode() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    let instruction = Instruction {
        operation: Operation::PHA,
        addressing: AddressMode::Immediate,
        cycles: 3,
    };

    let result = cpu.stack_operations(&instruction);

    assert!(result.is_err());
}

#[test]
fn test_nop_does_nothing() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    // Seed CPU with non-zero state
    cpu.A = 0x42;
    cpu.X = 0x24;
    cpu.Y = 0x11;
    cpu.P = 0b1010_1010;
    cpu.SP = 0xEE;

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xEA) // NOP
        .unwrap();

    cpu.nop_operation(instruction).unwrap();

    assert_eq!(cpu.A, 0x42);
    assert_eq!(cpu.X, 0x24);
    assert_eq!(cpu.Y, 0x11);
    assert_eq!(cpu.P, 0b1010_1010);
    assert_eq!(cpu.SP, 0xEE);
}

#[test]
fn test_nop_sets_cycles() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    let instruction = opcode_lookup::OPCODE_LOOKUP
        .get(&0xEA) // NOP
        .unwrap();

    cpu.nop_operation(instruction).unwrap();

    assert_eq!(cpu.cycles_remaining, instruction.cycles);
}

#[test]
fn test_nop_invalid_address_mode() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    let instruction = Instruction {
        operation: Operation::NOP,
        addressing: AddressMode::Immediate,
        cycles: 2,
    };

    let result = cpu.nop_operation(&instruction);
    assert!(result.is_err());
}

#[test]
fn test_nop_invalid_operation() {
    let bus = MockBus::new();
    let mut cpu = CPU::<MockBus>::new(bus);

    let instruction = Instruction {
        operation: Operation::LDA, // wrong op
        addressing: AddressMode::Implicit,
        cycles: 2,
    };

    let result = cpu.nop_operation(&instruction);
    assert!(result.is_err());
}
