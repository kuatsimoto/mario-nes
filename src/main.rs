mod rom_loader;
mod mapper;

fn main() {
    let _result = match rom_loader::load_rom() {
        Ok(cartridge) => cartridge,
        Err(error) => panic!("{}", error)
    };
}
