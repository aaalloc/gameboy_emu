use std::fs::File;
use std::io::Read;

use log::{debug, info};

#[repr(usize)]
enum Address {
    ROMSize = 0x148,
    CartridgeType = 0x147,
    HeaderCheckSum = 0x14D,
}

#[warn(non_snake_case)]
mod AddressRanges {
    use std::ops::RangeInclusive;
    pub const TITLE: RangeInclusive<u16> = 0x0134..=0x0143;
    pub const CHECKSUM: RangeInclusive<u16> = 0x0134..=0x014C;
}

impl From<Address> for usize {
    fn from(a: Address) -> usize {
        a as usize
    }
}
pub trait Cartdrige: Send {
    fn read(&self, address: u16) -> u8;
    fn set(&mut self, address: u16, value: u8);

    fn ensure_nintendo_logo(&self) {
        const NINTENDO_LOGO: [u8; 48] = [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C,
            0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6,
            0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC,
            0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
        ];
        for i in NINTENDO_LOGO.iter().enumerate() {
            if self.read(0x0104 + i.0 as u16) != *i.1 {
                panic!("Invalid Nintendo logo");
            }
        }
        debug!("Nintendo logo is valid");
    }

    fn ensure_header_checksum(&self) {
        let mut checksum: u8 = 0;
        for i in AddressRanges::CHECKSUM {
            checksum = checksum.wrapping_sub(self.read(i)).wrapping_sub(1);
        }
        if checksum != self.read(Address::HeaderCheckSum as u16) {
            panic!(
                "Invalid header checksum, expected {:#04x} but got {:#04x}",
                checksum,
                self.read(Address::HeaderCheckSum as u16)
            );
        }
        debug!("Header checksum is valid");
    }

    fn get_title(&self) -> String {
        let mut title = String::new();
        for i in AddressRanges::TITLE {
            match self.read(i) {
                0 => break,
                c => title.push(c as char),
            }
        }
        title
    }
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
    let res: Box<dyn Cartdrige>;
    match rom[Address::CartridgeType as usize] {
        0x00 => {
            res = Box::new(RomOnly(rom));
        }
        _ => {
            panic!(
                "Unsupported cartdrige type: {:#04x}",
                rom[Address::CartridgeType as usize]
            );
        }
    }
    res.ensure_nintendo_logo();
    res.ensure_header_checksum();
    info!("ROM title: {}", res.get_title());
    res
}

impl Cartdrige for RomOnly {
    fn read(&self, address: u16) -> u8 {
        self.0[address as usize]
    }
    fn set(&mut self, _address: u16, _value: u8) {
        panic!("Cannot write to ROM");
    }
}
