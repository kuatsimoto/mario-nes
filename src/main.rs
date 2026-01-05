mod rom_loader;
mod mapper;
mod cpu_bus;

fn main() {
    let _result = match rom_loader::load_rom() {
        Ok(cartridge) => cartridge,
        Err(error) => panic!("{}", error)
    };
}
