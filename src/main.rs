use chip8_emu::chip8::Chip8;

fn main() {
    println!("CHIP-8 emulator starting...");
    let mut cpu = Chip8::new();

    let sample_rom: Vec<u8> = vec![0x12, 0x00, 0xAB, 0xFF];
    match cpu.load_rom(&sample_rom) {
        Ok(()) => println!("ROM loaded succesfully!"),
        Err(e) => println!("Error loading ROM: {}", e),
    }
}
