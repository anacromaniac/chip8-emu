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

pub enum Instruction {
    /// 0NNN - SYS - Jump to a machine code routine at nnn
    /// Ignored in modern interpreters so it won't do anything
    Sys { addr: u16 },
    /// 00E0 - CLS - Clear the display
    Cls,
    /// 00EE - RET - Return from subroutine
    Ret,
    /// 1NNN - JP addr - Jump to location nnn
    Jp { addr: u16 },
    /// 2NNN - CALL addr - Call subroutine at nnn
    Call { addr: u16 },
    /// 6XKK - LD Vx, byte - The interpreter puts the value kk into register Vx
    LdVxByte { x: usize, kk: u8 },
    /// 7XKK - ADD Vx, byte - Adds the value kk to the value of register Vx, then stores the result in Vx
    AddVxByte { x: usize, kk: u8 },
    /// ANNN - LD I, addr - The value of register I is set to nnn
    LdI { nnn: u16 },
    /// Unknown opcode
    Unknown(u16),
}

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

    pub fn decode(&self, opcode: u16) -> Instruction {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n   = (opcode & 0x000F) as u8;
        let kk  = (opcode & 0x00FF) as u8;
        let nnn= opcode & 0x0FFF;

        match (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            opcode & 0x000F,
        ) {
            (0x0, 0x0, 0xE, 0x0) => Instruction::Cls,
            (0x0, 0x0, 0xE, 0xE) => Instruction::Ret,
            (0x0, _, _, _)       => Instruction::Sys { addr: nnn },
            (0x1, _, _, _)       => Instruction::Jp { addr: nnn },
            (0x2, _, _, _)       => Instruction::Call { addr: nnn },
            (0x6, _, _, _)       => Instruction::LdVxByte { x, kk },
            (0x7, _, _, _)       => Instruction::AddVxByte { x, kk },
            (0xA, _, _, _)       => Instruction::LdI { nnn },
            _                    => Instruction::Unknown(opcode),
        }
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Sys { addr: _ } => {
                // ignored
            }

            Instruction::Cls => {
                self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
            }

            Instruction::Ret => {
                self.pc = self.stack.pop().expect("RET called with empty stack");
            }

            Instruction::Jp { addr } => {
                self.pc = addr;
            }

            Instruction::Call { addr } => {
                self.stack.push(self.pc);
                self.pc = addr;
            }

            Instruction::LdVxByte { x, kk } => {
                self.v[x] = kk;
            }

            Instruction::AddVxByte { x, kk } => {
                self.v[x] = self.v[x].wrapping_add(kk);
            }

            Instruction::LdI { nnn } => {
                self.i = nnn;
            }

            Instruction::Unknown(opcode) => {
                eprintln!("Unknown opcode: {:#06X}", opcode);
            }
        }
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

    mod opcode_execute {
        use super::*;

        #[test]
        fn test_opcode_cls_clears_display() {
            let mut cpu = Chip8::new();
            cpu.display[0] = true;
            cpu.display[100] = true;
            let instruction = cpu.decode(0x00E0);
            cpu.execute(instruction);
            assert!(cpu.display.iter().all(|&p| !p))
        }

        #[test]
        fn test_opcode_jp_sets_pc() {
            let mut cpu = Chip8::new();
            cpu.execute(cpu.decode(0x1ABC));
            assert_eq!(cpu.pc, 0xABC);
        }

        #[test]
        fn test_opcode_ld_vx_sets_register() {
            let mut cpu = Chip8::new();
            // 6XKK → sets V3 = 0x42
            cpu.execute(cpu.decode(0x6342));
            assert_eq!(cpu.v[3], 0x42);
        }

        #[test]
        fn test_opcode_add_vx_adds_value() {
            let mut cpu = Chip8::new();
            cpu.v[2] = 10;
            // 7XKK → V2 += 5
            cpu.execute(cpu.decode(0x7205));
            assert_eq!(cpu.v[2], 15);
        }

        #[test]
        fn test_opcode_ld_i_sets_i() {
            let mut cpu = Chip8::new();
            cpu.execute(cpu.decode(0xA123));
            assert_eq!(cpu.i, 0x123);
        }

        #[test]
        fn test_unknown_opcode_does_not_panic() {
            let mut cpu = Chip8::new();
            cpu.execute(Instruction::Unknown(0xFFFF));
        }

        #[test]
        fn test_call_pushes_pc_to_stack() {
            let mut cpu = Chip8::new();
            cpu.pc = 0x200;
            cpu.execute(Instruction::Call { addr: 0x300 });
            assert_eq!(cpu.stack.last().copied(), Some(0x200));
        }

        #[test]
        fn test_call_sets_pc_to_addr() {
            let mut cpu = Chip8::new();
            cpu.execute(Instruction::Call { addr: 0x300 });
            assert_eq!(cpu.pc, 0x300);
        }

        #[test]
        fn test_ret_restores_pc_from_stack() {
            let mut cpu = Chip8::new();
            cpu.stack.push(0x200);
            cpu.execute(Instruction::Ret);
            assert_eq!(cpu.pc, 0x200);
        }

        #[test]
        fn test_ret_pops_stack() {
            let mut cpu = Chip8::new();
            cpu.stack.push(0x200);
            cpu.execute(Instruction::Ret);
            assert!(cpu.stack.is_empty());
        }

        #[test]
        fn test_call_and_ret_roundtrip() {
            let mut cpu = Chip8::new();
            cpu.pc = 0x200;
            cpu.execute(Instruction::Call { addr: 0x300 });
            assert_eq!(cpu.pc, 0x300);
            cpu.execute(Instruction::Ret);
            assert_eq!(cpu.pc, 0x200);
        }

        #[test]
        #[should_panic(expected = "RET called with empty stack")]
        fn test_ret_empty_stack_panics() {
            let mut cpu = Chip8::new();
            cpu.execute(Instruction::Ret);
        }

        #[test]
        fn test_sys_is_ignored() {
            let mut cpu = Chip8::new();
            let pc_before = cpu.pc;
            cpu.execute(cpu.decode(0x0123));
            assert_eq!(cpu.pc, pc_before);
        }
    }
}
