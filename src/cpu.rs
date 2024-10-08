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
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.pc.increment();
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
            (
                0x60,
                Instruction {
                    opcode: 0x60,
                    mnemonic: "LD H,B",
                    length: 1,
                    cycles: 4,
                    execute: |cpu: &mut Cpu| {
                        cpu.registers.h = cpu.registers.b;
                        cpu.registers.pc.increment();
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
                        cpu.registers.pc.increment();
                    },
                },
            ),
        ]);
        m
    };
}

impl Cpu {
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

    pub fn step(&mut self) {
        let opcode = self.cartdrige.read(self.registers.pc.value());
        let instruction = INSTRUCTION_MAP
            .get(&opcode)
            .expect(format!("Unknown opcode: {:#04x}", opcode).as_str());
        (instruction.execute)(self);
        debug!("Opcode: {:#04x}", opcode);
        debug!("Instruction: {:?}", instruction.mnemonic);
        debug!("Registers: {:#?}", self.registers);
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
        cpu.step();
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
        todo!();
    }
}
