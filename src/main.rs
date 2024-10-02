mod cpu;
mod register;
mod window;
mod cartdrige;

use std::env;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];

    // window::sdl2_example();
    let rom = cartdrige::load(rom_path);
}