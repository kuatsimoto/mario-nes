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
    let instruction = opcode_lookup::OPCODE_LOOKUP.get(&opcode).expect("Invalid instruction");
    cpu.clear_flag_operation(instruction).expect("CLC operatio failed");
    assert!(!cpu.get_flag(CPU::<MockBus>::CARRY))
}
