mod chip8;

fn main() {
    println!("CHIP-8 emulator starting...");
    let mut cpu = chip8::Chip8::new();

    let sample_rom: Vec<u8> = vec![0x12, 0x00, 0xAB, 0xFF];
    cpu.load_rom(&sample_rom);

    println!("ROM loaded!");
}
