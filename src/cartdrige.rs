use std::fs::File;
use std::io::Read;

#[repr(usize)]
pub enum Address {
    ROMSize = 0x148,
    CartridgeType = 0x147,
}

impl From<Address> for usize {
    fn from(a: Address) -> usize {
        a as usize
    }
}

pub trait Cartdrige: Send {
    fn read(&self, address: u16) -> u8;
    fn set(&mut self, address: u16, value: u8);
}

pub struct RomOnly(pub Vec<u8>);

fn rom_size(rom_max: usize) -> usize {
    let v = 16384; // 32 KiB / 2
    match rom_max {
        // 32 KiB × (1 << <value>)
        0x00..=0x08 => 2 * v * (1 << rom_max),
        0x52 => 72 * v,
        0x53 => 80 * v,
        0x54 => 96 * v,
        _ => panic!("Invalid ROM size: {:#04x}", rom_max),
    }
}

pub fn load(path: &str) -> Box<dyn Cartdrige> {
    let mut rom = Vec::new();
    let mut f = File::open(path).unwrap();
    f.read_to_end(&mut rom).unwrap();
    if rom.len() < 0x0150 {
        panic!("ROM is too small: {:#06x}", rom.len());
    }
    let rom_size = rom_size(rom[Address::ROMSize as usize] as usize);
    if rom.len() > rom_size {
        panic!("ROM size is bigger than expected: {:#06x}", rom.len());
    }

    // Cartdrige type
    match rom[Address::CartridgeType as usize] { 
        0x00 => {
            Box::new(RomOnly(rom))
        },
        _ => {
            panic!("Unsupported cartdrige type: {:#04x}", rom[Address::CartridgeType as usize]);
        }
    }
}

impl Cartdrige for RomOnly {
    fn read(&self, address: u16) -> u8 {
        self.0[address as usize]
    }
    fn set(&mut self, _address: u16, _value: u8) {
        panic!("Cannot write to ROM");
    }
}