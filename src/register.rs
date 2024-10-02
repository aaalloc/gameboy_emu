use bitflags::bitflags;

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Registers {
    pub a: u8, // Accumulator
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: StackPointer,
    pub pc: ProgramCounter,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StackPointer(pub u16);
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ProgramCounter(pub u16);
impl ProgramCounter {
    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct Flags: u8 {
        // set only if the result of the operation is zero
        // used for conditional jumps
        const ZERO = 1 << 7;
        const SUBTRACTION =  1 << 6;
        const HALFCARRY =  1 << 5;
        // set in these cases
        // When the result of an 8-bit addition is higher than $FF.
        // When the result of a 16-bit addition is higher than $FFFF.
        // When the result of a subtraction or comparison is lower than zero (like in Z80 and x86 CPUs, but unlike in 65XX and ARM CPUs).
        // When a rotate/shift operation shifts out a '1' bit.
        const CARRY = 1 << 4;
    }
}
