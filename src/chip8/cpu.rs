//! CHIP-8 Emulator
//!
//! Technical reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
//! Guide: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
//! Test suite: https://github.com/Timendus/chip8-test-suite
//!
const MEMORY_SIZE: usize = 4096;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;
const ROM_START: u16 = 0x200;

// 0-F hexadecimal digits
const FONT_CHARS: usize = 16;
const FONT_BYTES_PER_CHAR: usize = 5;
const FONTSET_SIZE: usize = FONT_CHARS * FONT_BYTES_PER_CHAR;
const FONTSET_START: usize = 0x000;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    // 4KB RAM
    pub(crate) memory: [u8; MEMORY_SIZE],
    // V0 to VF registers
    v: [u8; NUM_REGISTERS],

    // Program Counter
    pub(crate) pc: u16,

    // Index Register
    i: u16,

    stack: Vec<u16>,

    // 64x32 display, true = pixel on
    display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],

    // 60hz timers
    delay_timer: u8,
    sound_timer: u8,

    //16 keys, true = pressed
    keys: [bool; NUM_KEYS]
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            pc: ROM_START,
            i: 0,
            stack: Vec::new(),
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; NUM_KEYS]
        };

        chip8.memory[FONTSET_START..FONTSET_START + FONTSET_SIZE].copy_from_slice(&FONTSET);

        chip8
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), String> {
        let max_size = MEMORY_SIZE - ROM_START as usize;

        if data.len() > max_size {
            return Err(format!(
                "ROM too large: {} bytes, maximum {} bytes",
                data.len(),
                max_size)
            );
        }

        let start = ROM_START as usize;
        let end = start + data.len();

        self.memory[start..end].copy_from_slice(data);

        Ok(())
    }

    pub fn fetch(&mut self) -> u16 {
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte = self.memory[self.pc as usize + 1] as u16;
        let opcode = (high_byte << 8) | low_byte;
        self.pc += 2;

        opcode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pc_starts_at_rom_start() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.pc, ROM_START);
    }

    #[test]
    fn test_new_memory_is_zeroed() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.memory[ROM_START as usize], 0);
    }

    #[test]
    fn test_load_rom_ok() {
        let mut cpu = Chip8::new();
        let rom = vec![0x43, 0x6F, 0x77, 0x67, 0x6F, 0x64];
        let result = cpu.load_rom(&rom);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_rom_data_in_memory() {
        let mut cpu = Chip8::new();
        let rom = vec![0x43, 0x6F, 0x77, 0x67, 0x6F, 0x64];
        cpu.load_rom(&rom).unwrap();
        assert_eq!(cpu.memory[0x200], 0x43);
        assert_eq!(cpu.memory[0x201], 0x6F);
        assert_eq!(cpu.memory[0x202], 0x77);
        assert_eq!(cpu.memory[0x203], 0x67);
        assert_eq!(cpu.memory[0x204], 0x6F);
        assert_eq!(cpu.memory[0x205], 0x64);
    }

    #[test]
    fn test_load_rom_too_large() {
        let mut cpu = Chip8::new();
        let rom = vec![0u8; 4000];
        let result = cpu.load_rom(&rom);
        assert!(result.is_err());
    }

    #[test]
    fn test_fontset_loaded_at_start() {
        let cpu = Chip8::new();
        // "0" starts at 0x000, first byte is 0xF0
        assert_eq!(cpu.memory[0x000], 0xF0);
        // "1" starts at 0x005, first byte is 0x20
        assert_eq!(cpu.memory[0x005], 0x20);
        // "F" starts at 0x04B (75), first byte is 0xF0
        assert_eq!(cpu.memory[0x04B], 0xF0);
    }

    #[test]
    fn test_fontset_not_overwritten_by_rom() {
        let mut cpu = Chip8::new();
        let rom = vec![0x12, 0x00];
        cpu.load_rom(&rom).unwrap();
        assert_eq!(cpu.memory[0x000], 0xF0);
    }

    #[test]
    fn test_fetch_reads_two_bytes() {
        let mut cpu = Chip8::new();
        cpu.memory[0x200] = 0x12;
        cpu.memory[0x201] = 0x00;

        let opcode = cpu.fetch();

        assert_eq!(opcode, 0x1200);
    }

    #[test]
    fn test_fetch_advances_pc() {
        let mut cpu = Chip8::new();
        cpu.memory[0x200] = 0x12;
        cpu.memory[0x201] = 0x00;

        cpu.fetch();

        assert_eq!(cpu.pc, 0x202);
    }
}
