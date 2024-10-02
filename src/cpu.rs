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

    pub fn cpu_step(&mut self) {
        match self.cartdrige.read(self.registers.pc.value()) {
            0x00 => {
                // NOP
                self.registers.pc.increment();
            }
            _ => {
                panic!(
                    "Unimplemented opcode: {:#04x}",
                    self.cartdrige.read(self.registers.pc.value())
                );
            }
        }
    }
}
