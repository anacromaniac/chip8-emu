const MEMORY_SIZE: usize = 4096;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;
const ROM_START: u16 = 0x200;

pub struct Chip8 {
    // 4KB RAM
    memory: [u8; MEMORY_SIZE],
    // V0 to VF registers
    v: [u8; NUM_REGISTERS],

    // Program Counter
    pc: u16,

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
        Chip8 {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            pc: ROM_START,
            i: 0,
            stack: Vec::new(),
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; NUM_KEYS]
        }
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
}