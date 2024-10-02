mod cartdrige;
mod cpu;
mod register;
mod window;

use std::env;

use log::info;

pub fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    // set log level default to info
    info!("starting up");
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];
    // window::sdl2_example();
    let rom = cartdrige::load(rom_path);
}
