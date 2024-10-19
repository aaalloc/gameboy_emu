use lazy_static::lazy_static;
use log::debug;
use std::collections::HashMap;

/// Main logic for the CPU
/// Following
/// https://gbdev.io/pandocs/CPU_Registers_and_Flags.html#the-flags-register-lower-8-bits-of-af-register
use crate::{
    cartdrige::Cartdrige,
    register::{self, ProgramCounter, Registers, StackPointer},
};

pub struct Cpu {
    pub registers: Registers,
    pub cartdrige: Box<dyn Cartdrige>,
}

pub struct Instruction {
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub length: u8, // in bytes
    pub cycles: u8,
    pub execute: fn(&mut Cpu),
}

lazy_static! {
    pub static ref INSTRUCTION_MAP: HashMap<u8, Instruction> = {
        let m: HashMap<u8, Instruction> = HashMap::from([
            (
                0x00,
                Instruction {
                    opcode: 0x00,
                    mnemonic: "NOP",
                    length: 1,
                    cycles: 4,
                    execute: |_cpu: &mut Cpu| {},
                },
            ),
            (
                0x10,
                Instruction {
                    opcode: 0x10,
                    mnemonic: "STOP 0",
                    length: 2,
                    cycles: 4,
                    execute: |_cpu: &mut Cpu| {},
                },
            ),
            (
                0x20,
                Instruction {
                    opcode: 0x20,
                    mnemonic: "JR NZ,r8",
                    length: 2,
                    cycles: 8,
                    execute: |cpu: &mut Cpu| {
                        let offset = cpu.fetch() as i8;
                        if !cpu.registers.f.contains(register::Flags::ZERO) {
                            cpu.registers.pc.0 = (cpu.registers.pc.0 as i32 + offset as i32) as u16;
                        }
                    },
                },
            ),
            (
                0x60,
                Instruction {
                    opcode: 0x60,
                    mnemonic: "LD H,B",
                    length: 1,
                    cycles: 4,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.h = cpu.registers.b;
                        cpu.registers.pc.0 += 1;
                    },
                },
            ),
            (
                0x05,
                Instruction {
                    opcode: 0x05,
                    mnemonic: "DEC B",
                    length: 1,
                    cycles: 4,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.b = cpu.alu_dec(cpu.registers.b);
                    },
                },
            ),
            (
                0x06,
                Instruction {
                    opcode: 0x06,
                    mnemonic: "LD B,d8",
                    length: 2,
                    cycles: 8,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.b = cpu.fetch();
                    },
                },
            ),
            (
                0x0D,
                Instruction {
                    opcode: 0x0D,
                    mnemonic: "DEC C",
                    length: 1,
                    cycles: 4,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.c = cpu.alu_dec(cpu.registers.c);
                    },
                },
            ),
            (
                0x0E,
                Instruction {
                    opcode: 0x0E,
                    mnemonic: "LD C,d8",
                    length: 2,
                    cycles: 8,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.c = cpu.fetch();
                    },
                },
            ),
            (
                0x21,
                Instruction {
                    opcode: 0x21,
                    mnemonic: "LD HL,d16",
                    length: 3,
                    cycles: 12,
                    execute: |cpu: &mut Cpu| {
                        let word = cpu.fetch_word();
                        cpu.registers.h = (word >> 8) as u8;
                        cpu.registers.l = word as u8;
                    },
                },
            ),
            (
                0x2F,
                Instruction {
                    opcode: 0x2F,
                    mnemonic: "CPL",
                    length: 1,
                    cycles: 4,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.a = !cpu.registers.a;
                        cpu.registers.f.set(register::Flags::SUBTRACTION, true);
                        cpu.registers.f.set(register::Flags::HALFCARRY, true);
                        cpu.registers.pc.0 += 1;
                    },
                },
            ),
            (
                0x32,
                Instruction {
                    opcode: 0x32,
                    mnemonic: "LD (HL-),A",
                    length: 1,
                    cycles: 8,
                    execute: |cpu: &mut Cpu| {
                        let mut hl = (cpu.registers.h as u16) << 8 | cpu.registers.l as u16;
                        cpu.cartdrige.set(hl, cpu.registers.a);
                        hl -= 1;
                        cpu.registers.h = (hl >> 8) as u8;
                        cpu.registers.l = hl as u8;
                    },
                },
            ),
            (
                0x3E,
                Instruction {
                    opcode: 0x3E,
                    mnemonic: "LD A,d8",
                    length: 2,
                    cycles: 8,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.a = cpu.fetch();
                    },
                },
            ),
            (
                0xaf,
                Instruction {
                    opcode: 0xaf,
                    mnemonic: "XOR A, A",
                    length: 1,
                    cycles: 4,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.a ^= cpu.registers.a;
                        cpu.registers.f.set(register::Flags::ZERO, true);
                        cpu.registers.f.set(register::Flags::SUBTRACTION, false);
                        cpu.registers.f.set(register::Flags::HALFCARRY, false);
                        cpu.registers.f.set(register::Flags::CARRY, false);
                    },
                },
            ),
            (
                0xc3,
                Instruction {
                    opcode: 0xc3,
                    mnemonic: "JP a16",
                    length: 3,
                    cycles: 16,
                    execute: |cpu: &mut Cpu| {
                        let word = cpu.cartdrige.read_word(cpu.registers.pc.value());
                        cpu.registers.pc.0 = word;
                    },
                },
            ),
        ]);
        m
    };
}

impl Cpu {
    fn fetch(&mut self) -> u8 {
        let value = self.cartdrige.read(self.registers.pc.value());
        self.registers.pc.0 += 1;
        value
    }

    fn fetch_word(&mut self) -> u16 {
        let value = self.cartdrige.read_word(self.registers.pc.value());
        self.registers.pc.0 += 2;
        value
    }

    fn alu_dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.registers.f.set(register::Flags::ZERO, result == 0);
        self.registers.f.set(register::Flags::SUBTRACTION, true);
        // the idea here is to check if the lower 4 bits are 0
        // https://gist.github.com/meganesu/9e228b6b587decc783aa9be34ae27841
        self.registers
            .f
            .set(register::Flags::HALFCARRY, (value & 0x0F) == 0x0F);

        result
    }

    pub fn new(cartdrige: Box<dyn Cartdrige>) -> Self {
        Self {
            // Following DMG
            // https://gbdev.io/pandocs/Power_Up_Sequence.html?highlight=half#cpu-registers
            registers: Registers {
                a: 0x01,
                f: register::Flags::ZERO,
                b: 0x00,
                c: 0x13,
                d: 0,
                e: 0xD8,
                h: 0x01,
                l: 0x4D,
                sp: StackPointer(0xFFFE),
                pc: ProgramCounter(0x0100),
            },
            cartdrige,
        }
    }

    pub fn step(&mut self) -> &Instruction {
        let opcode = self.fetch();
        let instruction = INSTRUCTION_MAP
            .get(&opcode)
            .expect(format!("Unknown opcode: {:#04x}", opcode).as_str());
        (instruction.execute)(self);
        debug!("Opcode: {:#04x}", opcode);
        debug!("Instruction: {:?}", instruction.mnemonic);
        debug!("Registers: {:#?}", self.registers);
        instruction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartdrige::RomOnly;

    #[test]
    fn test_cpu_step() {
        let mut cpu = Cpu::new(Box::new(RomOnly(vec![0x00; 0x101])));
        cpu.step();
        assert_eq!(cpu.registers.pc.value(), 0x0101);
    }

    #[test]
    fn test_cpu_step_nop() {
        let mut cpu = Cpu::new(Box::new(RomOnly(vec![0x00; 0x101])));
        let tmp_registers = cpu.registers;
        let instruction = cpu.step();
        assert_eq!(instruction.mnemonic, "NOP");
        assert_eq!(cpu.registers.pc.value(), 0x0101);
        assert_eq!(
            cpu.registers,
            Registers {
                pc: ProgramCounter(0x0101),
                ..tmp_registers
            }
        );
    }

    #[test]
    fn test_cpu_step_jp_a16() {
        let mut fake_rom_data = vec![0x00; 0xFFF];
        fake_rom_data[0x100] = 0xc3; // JP a16
        fake_rom_data[0x101] = 0xFF; // value to jump
        let mut cpu = Cpu::new(Box::new(RomOnly(fake_rom_data)));
        let tmp_registers = cpu.registers;
        let instruction = cpu.step();
        assert_eq!(instruction.mnemonic, "JP a16");
        assert_eq!(cpu.registers.pc.value(), 0xFF);
        assert_eq!(
            cpu.registers,
            Registers {
                pc: ProgramCounter(0xFF),
                ..tmp_registers
            }
        );
    }

    #[test]
    fn test_cpu_step_xor_a_a() {
        let mut fake_rom_data = vec![0x00; 0x101];
        fake_rom_data[0x100] = 0xAF; // XOR A, A
        let mut cpu = Cpu::new(Box::new(RomOnly(fake_rom_data)));
        let tmp_registers = cpu.registers;
        let instruction = cpu.step();
        assert_eq!(instruction.mnemonic, "XOR A, A");
        assert_eq!(cpu.registers.pc.value(), 0x101);
        assert_eq!(
            cpu.registers,
            Registers {
                a: 0x00,
                f: register::Flags::ZERO,
                pc: ProgramCounter(0x101),
                ..tmp_registers
            }
        );
    }

    #[test]
    fn test_cpu_step_ld_hl_d16() {
        let mut fake_rom_data = vec![0x00; 0x111];
        fake_rom_data[0x100] = 0x21; // LD HL,d16
        fake_rom_data[0x102] = 0x12; // H register value
        fake_rom_data[0x101] = 0x34; // L register value
        let mut cpu = Cpu::new(Box::new(RomOnly(fake_rom_data)));
        let tmp_registers = cpu.registers;
        let instruction = cpu.step();
        assert_eq!(instruction.mnemonic, "LD HL,d16");
        assert_eq!(cpu.registers.pc.value(), 0x103);
        assert_eq!(
            cpu.registers,
            Registers {
                h: 0x12,
                l: 0x34,
                pc: ProgramCounter(0x103),
                ..tmp_registers
            }
        );
    }

    #[test]
    fn test_cpu_step_ld_c_d8() {
        let mut fake_rom_data = vec![0x00; 0x102];
        fake_rom_data[0x100] = 0x0E; // LD C,d8
        fake_rom_data[0x101] = 0x12; // C register value
        let mut cpu = Cpu::new(Box::new(RomOnly(fake_rom_data)));
        let tmp_registers = cpu.registers;
        let instruction = cpu.step();
        assert_eq!(instruction.mnemonic, "LD C,d8");
        assert_eq!(cpu.registers.pc.value(), 0x102);
        assert_eq!(
            cpu.registers,
            Registers {
                c: 0x12,
                pc: ProgramCounter(0x102),
                ..tmp_registers
            }
        );
    }

    #[test]
    fn test_cpu_step_ld_b_d8() {
        let mut fake_rom_data = vec![0x00; 0x102];
        fake_rom_data[0x100] = 0x06; // LD B,d8
        fake_rom_data[0x101] = 0x12; // B register value
        let mut cpu = Cpu::new(Box::new(RomOnly(fake_rom_data)));
        let tmp_registers = cpu.registers;
        let instruction = cpu.step();
        assert_eq!(instruction.mnemonic, "LD B,d8");
        assert_eq!(cpu.registers.pc.value(), 0x102);
        assert_eq!(
            cpu.registers,
            Registers {
                b: 0x12,
                pc: ProgramCounter(0x102),
                ..tmp_registers
            }
        );
    }
}
