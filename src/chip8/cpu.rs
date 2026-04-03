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
const STACK_SIZE: usize = 16;

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

#[derive(Debug, PartialEq)]
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
    LdI { addr: u16 },

    /// 3XKK - SE Vx, byte
    /// Skip next instruction if Vx == kk
    Se { x: usize, kk: u8 },

    /// 4XKK - SNE Vx, byte
    /// Skip next instruction if Vx != kk
    Sne { x: usize, kk: u8 },

    /// 5XY0 - SE Vx, Vy
    /// Skip next instruction if Vx == Vy
    SeVxVy { x: usize, y: usize },

    /// 8XY0 - LD Vx, Vy
    /// Set Vx = Vy
    LdVxVy { x: usize, y: usize },

    /// 8XY1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy
    OrVxVy { x: usize, y: usize },

    /// 8XY2 - AND Vx, Vy
    /// Set Vx = Vx AND Vy
    AndVxVy { x: usize, y: usize },

    /// 8XY3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy
    XorVxVy { x: usize, y: usize },

    /// 8XY4 - ADD Vx, Vy
    /// Set Vx = Vx + Vy, VF = carry
    AddVxVy { x: usize, y: usize },

    /// 8XY5 - SUB Vx, Vy
    /// Set Vx = Vx - Vy, VF = NOT borrow (1 if Vx > Vy, 0 otherwise)
    SubVxVy { x: usize, y: usize },

    /// 8XY6 - SHR Vx, Vy
    /// Set Vx = Vy >> 1, VF = least significant bit of Vy before shift
    ShrVx { x: usize, y: usize },

    /// 8XY7 - SUBN Vx, Vy
    /// Set Vx = Vy - Vx, VF = NOT borrow (1 if Vy > Vx, 0 otherwise)
    SubnVxVy { x: usize, y: usize },

    /// 8XYE - SHL Vx, Vy
    /// Set Vx = Vy << 1, VF = most significant bit of Vy before shift
    ShlVx { x: usize, y: usize },

    /// 9XY0 - SNE Vx, Vy
    /// Skip next instruction if Vx != Vy
    SneVxVy { x: usize, y: usize },

    /// BNNN - JP V0, addr
    /// Jump to location NNN + V0
    JpV0 { addr: u16 },

    /// CXKK - RND Vx, byte
    /// Set Vx = random byte AND kk
    Rnd { x: usize, kk: u8 },

    /// DXYN - DRW Vx, Vy, nibble
    /// Display n-byte sprite starting at memory location I at (Vx, Vy)
    /// Set VF = collision
    Drw { x: usize, y: usize, n: u8 },

    /// EX9E - SKP Vx
    /// Skip next instruction if key with value Vx is pressed
    Skp { x: usize },

    /// EXA1 - SKNP Vx
    /// Skip next instruction if key with value Vx is not pressed
    Sknp { x: usize },

    /// FX07 - LD Vx, DT
    /// Set Vx = delay timer value
    LdVxDt { x: usize },

    /// FX15 - LD DT, Vx
    /// Set delay timer = Vx
    LdDtVx { x: usize },

    /// FX18 - LD ST, Vx
    /// Set sound timer = Vx
    LdStVx { x: usize },

    /// FX1E - ADD I, Vx
    /// Set I = I + Vx
    AddIVx { x: usize },

    /// FX29 - LD F, Vx
    /// Set I = location of sprite for digit Vx
    LdFVx { x: usize },

    /// FX0A - LD Vx, K
    /// Wait for key press, store key value in Vx
    /// Execution stops until a key is pressed
    LdVxK { x: usize },

    /// FX33 - LD B, Vx
    /// Store BCD representation of Vx in memory at I, I+1, I+2
    LdBVx { x: usize },

    /// FX55 - LD [I], Vx
    /// Store registers V0 through Vx in memory starting at I
    LdIVx { x: usize },

    /// FX65 - LD Vx, [I]
    /// Read registers V0 through Vx from memory starting at I
    LdVxI { x: usize },

    /// Unknown opcode
    Unknown(u16),
}

/// Configures behavioral quirks of the CHIP-8 CPU.
///
/// Each flag controls one behavioral difference between the original 1977
/// COSMAC VIP interpreter and modern CHIP-48/SCHIP-era interpreters.
///
/// Use [`Chip8Config::default()`] for modern behavior or
/// [`Chip8Config::legacy()`] for original 1977 behavior.
#[derive(Debug, PartialEq)]
pub struct Chip8Config {
    /// Shift source register for 8XY6 (SHR) and 8XYE (SHL).
    ///
    /// - `true` (legacy): reads from **Vy**, shifts it, stores result in Vx;
    ///   VF = lost bit of Vy.
    /// - `false` (modern/default): shifts **Vx** in-place, Vy is ignored;
    ///   VF = lost bit of Vx.
    pub shift_uses_vy: bool,

    /// Whether FX55 (store) and FX65 (load) advance the I register.
    ///
    /// - `true` (legacy): I is incremented by X+1 after the operation.
    /// - `false` (modern/default): I is left unchanged.
    pub load_store_increments_i: bool,

    /// Offset register used by the BNNN / BXNN jump instruction.
    ///
    /// - `false` (legacy): PC = NNN + **V0**.
    /// - `true` (modern/default): PC = XNN + **VX** (X is nibble 2 of the
    ///   opcode).
    pub jump_v0_uses_vx: bool,

    /// Sprite drawing behavior when pixels reach the screen edge (DXYN).
    ///
    /// - `true` (legacy): pixels beyond the screen boundaries are **clipped**
    ///   (not drawn).
    /// - `false` (modern/default): pixels **wrap** around to the opposite edge.
    pub clip_sprites: bool,

    /// VF register behavior after logical OR / AND / XOR (8XY1 / 8XY2 / 8XY3).
    ///
    /// - `true` (legacy): VF is **reset to 0** after the operation.
    /// - `false` (modern/default): VF is **unchanged** after the operation.
    pub reset_vf_after_logical: bool,
}

impl Default for Chip8Config {
    /// Returns the modern (CHIP-48/SCHIP era) configuration.
    fn default() -> Self {
        Self {
            shift_uses_vy: false,
            load_store_increments_i: false,
            jump_v0_uses_vx: true,
            clip_sprites: false,
            reset_vf_after_logical: false,
        }
    }
}

impl Chip8Config {
    /// Returns a configuration matching the original 1977 COSMAC VIP interpreter.
    pub fn legacy() -> Self {
        Self {
            shift_uses_vy: true,
            load_store_increments_i: true,
            jump_v0_uses_vx: false,
            clip_sprites: true,
            reset_vf_after_logical: true,
        }
    }
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
    keys: [bool; NUM_KEYS],

    config: Chip8Config,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new(Chip8Config::default())
    }
}

impl Chip8 {
    pub fn new(config: Chip8Config) -> Self {
        let mut chip8 = Chip8 {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            pc: ROM_START,
            i: 0,
            stack: Vec::new(),
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; NUM_KEYS],
            config,
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
                max_size
            ));
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
        let n = (opcode & 0x000F) as u8;
        let kk = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            opcode & 0x000F,
        ) {
            (0x0, 0x0, 0xE, 0x0) => Instruction::Cls,
            (0x0, 0x0, 0xE, 0xE) => Instruction::Ret,
            (0x0, _, _, _) => Instruction::Sys { addr: nnn },
            (0x1, _, _, _) => Instruction::Jp { addr: nnn },
            (0x2, _, _, _) => Instruction::Call { addr: nnn },
            (0x3, _, _, _) => Instruction::Se { x, kk },
            (0x4, _, _, _) => Instruction::Sne { x, kk },
            (0x5, _, _, 0x0) => Instruction::SeVxVy { x, y },
            (0x6, _, _, _) => Instruction::LdVxByte { x, kk },
            (0x7, _, _, _) => Instruction::AddVxByte { x, kk },
            (0x8, _, _, 0x0) => Instruction::LdVxVy { x, y },
            (0x8, _, _, 0x1) => Instruction::OrVxVy { x, y },
            (0x8, _, _, 0x2) => Instruction::AndVxVy { x, y },
            (0x8, _, _, 0x3) => Instruction::XorVxVy { x, y },
            (0x8, _, _, 0x4) => Instruction::AddVxVy { x, y },
            (0x8, _, _, 0x5) => Instruction::SubVxVy { x, y },
            (0x8, _, _, 0x6) => Instruction::ShrVx { x, y },
            (0x8, _, _, 0x7) => Instruction::SubnVxVy { x, y },
            (0x8, _, _, 0xE) => Instruction::ShlVx { x, y },
            (0x9, _, _, 0x0) => Instruction::SneVxVy { x, y },
            (0xA, _, _, _) => Instruction::LdI { addr: nnn },
            (0xB, _, _, _) => Instruction::JpV0 { addr: nnn },
            (0xC, _, _, _) => Instruction::Rnd { x, kk },
            (0xD, _, _, _) => Instruction::Drw { x, y, n },
            (0xE, _, 0x9, 0xE) => Instruction::Skp { x },
            (0xE, _, 0xA, 0x1) => Instruction::Sknp { x },
            (0xF, _, 0x0, 0x7) => Instruction::LdVxDt { x },
            (0xF, _, 0x0, 0xA) => Instruction::LdVxK { x },
            (0xF, _, 0x1, 0x5) => Instruction::LdDtVx { x },
            (0xF, _, 0x1, 0x8) => Instruction::LdStVx { x },
            (0xF, _, 0x1, 0xE) => Instruction::AddIVx { x },
            (0xF, _, 0x2, 0x9) => Instruction::LdFVx { x },
            (0xF, _, 0x3, 0x3) => Instruction::LdBVx { x },
            (0xF, _, 0x5, 0x5) => Instruction::LdIVx { x },
            (0xF, _, 0x6, 0x5) => Instruction::LdVxI { x },
            _ => Instruction::Unknown(opcode),
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
                if self.stack.len() == STACK_SIZE {
                    panic!("CALL stack overflow");
                }
                self.stack.push(self.pc);
                self.pc = addr;
            }

            Instruction::LdVxByte { x, kk } => {
                self.v[x] = kk;
            }

            Instruction::AddVxByte { x, kk } => {
                self.v[x] = self.v[x].wrapping_add(kk);
            }

            Instruction::LdI { addr: nnn } => {
                self.i = nnn;
            }

            Instruction::Se { x, kk } => {
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }

            Instruction::Sne { x, kk } => {
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }

            Instruction::SeVxVy { x, y } => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }

            Instruction::LdVxVy { x, y } => {
                self.v[x] = self.v[y];
            }

            Instruction::OrVxVy { x, y } => {
                self.v[x] |= self.v[y];
                self.reset_vf_if_logical_quirk();
            }

            Instruction::AndVxVy { x, y } => {
                self.v[x] &= self.v[y];
                self.reset_vf_if_logical_quirk();
            }

            Instruction::XorVxVy { x, y } => {
                self.v[x] ^= self.v[y];
                self.reset_vf_if_logical_quirk();
            }

            Instruction::AddVxVy { x, y } => {
                let (result, carry) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = result;
                self.v[0xF] = carry as u8;
            }

            Instruction::SubVxVy { x, y } => {
                let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = result;
                self.v[0xF] = !borrow as u8; // NOT borrow
            }

            Instruction::ShrVx { x, y } => {
                let src = if self.config.shift_uses_vy { y } else { x };
                let lsb = self.v[src] & 0x1;
                self.v[x] = self.v[src] >> 1;
                self.v[0xF] = lsb;
            }

            Instruction::SubnVxVy { x, y } => {
                let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = result;
                self.v[0xF] = !borrow as u8; // NOT borrow
            }

            Instruction::ShlVx { x, y } => {
                let src = if self.config.shift_uses_vy { y } else { x };
                let msb = (self.v[src] >> 7) & 0x1;
                self.v[x] = self.v[src] << 1;
                self.v[0xF] = msb;
            }

            Instruction::SneVxVy { x, y } => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }

            Instruction::JpV0 { addr } => {
                let offset_reg = if self.config.jump_v0_uses_vx {
                    (addr >> 8) as usize
                } else {
                    0
                };
                self.pc = addr + self.v[offset_reg] as u16;
            }

            Instruction::Rnd { x, kk } => {
                let random: u8 = rand::random();
                self.v[x] = random & kk;
            }

            Instruction::Drw { x, y, n } => {
                let x_start = self.v[x] as usize % DISPLAY_WIDTH;
                let y_start = self.v[y] as usize % DISPLAY_HEIGHT;

                // reset collision flag
                self.v[0xF] = 0;

                for row in 0..n as usize {
                    let sprite_byte = self.memory[self.i as usize + row];

                    for col in 0..8 {
                        let sprite_pixel = (sprite_byte >> (7 - col)) & 0x1;

                        if sprite_pixel == 0 {
                            continue;
                        }

                        let px = x_start + col;
                        let py = y_start + row;
                        let (px, py) = if self.config.clip_sprites {
                            if px >= DISPLAY_WIDTH || py >= DISPLAY_HEIGHT {
                                continue;
                            }
                            (px, py)
                        } else {
                            (px % DISPLAY_WIDTH, py % DISPLAY_HEIGHT)
                        };
                        let idx = py * DISPLAY_WIDTH + px;

                        // collision detection
                        if self.display[idx] {
                            self.v[0xF] = 1;
                        }

                        self.display[idx] ^= true;
                    }
                }
            }

            Instruction::Skp { x } => {
                let key = self.v[x] as usize;
                if self.keys[key] {
                    self.pc += 2;
                }
            }

            Instruction::Sknp { x } => {
                let key = self.v[x] as usize;
                if !self.keys[key] {
                    self.pc += 2;
                }
            }

            Instruction::LdVxDt { x } => {
                self.v[x] = self.delay_timer;
            }

            Instruction::LdDtVx { x } => {
                self.delay_timer = self.v[x];
            }

            Instruction::LdStVx { x } => {
                self.sound_timer = self.v[x];
            }

            Instruction::AddIVx { x } => {
                self.i += self.v[x] as u16;
            }

            Instruction::LdFVx { x } => {
                // each font is 5 bytes, so digit Vx starts at FONTSET_START + Vx * 5
                self.i = (FONTSET_START + self.v[x] as usize * FONT_BYTES_PER_CHAR) as u16;
            }

            Instruction::LdVxK { x } => {
                // look for a pressed key
                let pressed = self.keys.iter().position(|&k| k);
                match pressed {
                    Some(key) => self.v[x] = key as u8,
                    // no key pressed — rewind PC to re-execute
                    None => self.pc -= 2,
                }
            }

            Instruction::LdBVx { x } => {
                let value = self.v[x];
                self.memory[self.i as usize] = value / 100; // hundreds
                self.memory[self.i as usize + 1] = (value / 10) % 10; // tens
                self.memory[self.i as usize + 2] = value % 10; // units
            }

            Instruction::LdIVx { x } => {
                for reg in 0..=x {
                    self.memory[self.i as usize + reg] = self.v[reg];
                }
                if self.config.load_store_increments_i {
                    self.i += (x + 1) as u16;
                }
            }

            Instruction::LdVxI { x } => {
                for reg in 0..=x {
                    self.v[reg] = self.memory[self.i as usize + reg];
                }
                if self.config.load_store_increments_i {
                    self.i += (x + 1) as u16;
                }
            }

            Instruction::Unknown(opcode) => {
                panic!("Unknown opcode: {:#06X}", opcode);
            }
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn is_beeping(&self) -> bool {
        self.sound_timer > 0
    }

    fn reset_vf_if_logical_quirk(&mut self) {
        if self.config.reset_vf_after_logical {
            self.v[0xF] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_chip8() -> Chip8 {
        Chip8::new(Chip8Config::default())
    }

    mod boot {
        use super::*;

        #[test]
        fn test_new_pc_starts_at_rom_start() {
            let chip8 = new_chip8();
            assert_eq!(chip8.pc, ROM_START);
        }

        #[test]
        fn test_rom_start_is_zeroed() {
            let chip8 = new_chip8();
            assert_eq!(chip8.memory[ROM_START as usize], 0);
        }
    }

    mod rom_loading {
        use super::*;

        #[test]
        fn test_load_rom_ok() {
            let mut cpu = new_chip8();
            let rom = vec![0x43, 0x6F, 0x77, 0x67, 0x6F, 0x64];
            let result = cpu.load_rom(&rom);
            assert!(result.is_ok());
        }

        #[test]
        fn test_load_rom_data_in_memory() {
            let mut cpu = new_chip8();
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
            let mut cpu = new_chip8();
            let rom = vec![0u8; MEMORY_SIZE - ROM_START as usize + 1];
            let result = cpu.load_rom(&rom);
            assert!(result.is_err());
        }
    }

    mod fontset {
        use super::*;

        #[test]
        fn test_fontset_loaded_at_start() {
            let cpu = new_chip8();
            // "0" starts at 0x000, first byte is 0xF0
            assert_eq!(cpu.memory[0x000], 0xF0);
            // "1" starts at 0x005, first byte is 0x20
            assert_eq!(cpu.memory[0x005], 0x20);
            // "F" starts at 0x04B (75), first byte is 0xF0
            assert_eq!(cpu.memory[0x04B], 0xF0);
        }

        #[test]
        fn test_fontset_not_overwritten_by_rom() {
            let mut cpu = new_chip8();
            let rom = vec![0x12, 0x00];
            cpu.load_rom(&rom).unwrap();
            assert_eq!(cpu.memory[0x000], 0xF0);
        }
    }

    mod fetch {
        use super::*;

        #[test]
        fn test_fetch_reads_two_bytes() {
            let mut cpu = new_chip8();
            cpu.memory[0x200] = 0x12;
            cpu.memory[0x201] = 0x00;
            assert_eq!(cpu.fetch(), 0x1200);
        }

        #[test]
        fn test_fetch_advances_pc() {
            let mut cpu = new_chip8();
            cpu.memory[0x200] = 0x12;
            cpu.memory[0x201] = 0x00;
            cpu.fetch();
            assert_eq!(cpu.pc, 0x202);
        }
    }

    mod decode {
        use super::*;

        #[test]
        fn test_decode_sys() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x0123), Instruction::Sys { addr: 0x123 });
        }

        #[test]
        fn test_decode_cls() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x00E0), Instruction::Cls);
        }

        #[test]
        fn test_decode_ret() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x00EE), Instruction::Ret);
        }

        #[test]
        fn test_decode_jp() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x1ABC), Instruction::Jp { addr: 0xABC });
        }

        #[test]
        fn test_decode_call() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x2ABC), Instruction::Call { addr: 0xABC });
        }

        #[test]
        fn test_decode_se_vx_byte() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x3210), Instruction::Se { x: 2, kk: 0x10 });
        }

        #[test]
        fn test_decode_sne_vx_byte() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x4210), Instruction::Sne { x: 2, kk: 0x10 });
        }

        #[test]
        fn test_decode_se_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x5230), Instruction::SeVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_ld_vx_byte() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x6342), Instruction::LdVxByte { x: 3, kk: 0x42 });
        }

        #[test]
        fn test_decode_add_vx_byte() {
            let cpu = new_chip8();
            assert_eq!(
                cpu.decode(0x7205),
                Instruction::AddVxByte { x: 2, kk: 0x05 }
            );
        }

        #[test]
        fn test_decode_ld_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8230), Instruction::LdVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_or_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8231), Instruction::OrVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_and_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8232), Instruction::AndVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_xor_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8233), Instruction::XorVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_add_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8234), Instruction::AddVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_sub_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8235), Instruction::SubVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_shr_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8236), Instruction::ShrVx { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_subn_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x8237), Instruction::SubnVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_shl_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x823E), Instruction::ShlVx { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_sne_vx_vy() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0x9230), Instruction::SneVxVy { x: 2, y: 3 });
        }

        #[test]
        fn test_decode_ld_i() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xA123), Instruction::LdI { addr: 0x123 });
        }

        #[test]
        fn test_decode_jp_v0() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xB123), Instruction::JpV0 { addr: 0x123 });
        }

        #[test]
        fn test_decode_rnd() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xC20F), Instruction::Rnd { x: 2, kk: 0x0F });
        }

        #[test]
        fn test_decode_drw() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xD125), Instruction::Drw { x: 1, y: 2, n: 5 });
        }

        #[test]
        fn test_decode_skp() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xE29E), Instruction::Skp { x: 2 });
        }

        #[test]
        fn test_decode_sknp() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xE2A1), Instruction::Sknp { x: 2 });
        }

        #[test]
        fn test_decode_ld_vx_dt() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF207), Instruction::LdVxDt { x: 2 });
        }

        #[test]
        fn test_decode_ld_vx_k() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF20A), Instruction::LdVxK { x: 2 });
        }

        #[test]
        fn test_decode_ld_dt_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF215), Instruction::LdDtVx { x: 2 });
        }

        #[test]
        fn test_decode_ld_st_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF218), Instruction::LdStVx { x: 2 });
        }

        #[test]
        fn test_decode_add_i_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF21E), Instruction::AddIVx { x: 2 });
        }

        #[test]
        fn test_decode_ld_f_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF229), Instruction::LdFVx { x: 2 });
        }

        #[test]
        fn test_decode_ld_b_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF233), Instruction::LdBVx { x: 2 });
        }

        #[test]
        fn test_decode_ld_i_vx() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF255), Instruction::LdIVx { x: 2 });
        }

        #[test]
        fn test_decode_ld_vx_i() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xF265), Instruction::LdVxI { x: 2 });
        }

        #[test]
        fn test_decode_unknown() {
            let cpu = new_chip8();
            assert_eq!(cpu.decode(0xFFFF), Instruction::Unknown(0xFFFF));
        }
    }

    mod execute {
        use super::*;

        mod sys {
            use super::*;

            #[test]
            fn test_sys_is_ignored() {
                let mut cpu = new_chip8();
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Sys { addr: 0x200 });
                assert_eq!(cpu.pc, pc_before);
            }
        }

        mod cls {
            use super::*;

            #[test]
            fn test_cls_clears_display() {
                let mut cpu = new_chip8();
                cpu.display[0] = true;
                cpu.display[100] = true;
                cpu.execute(Instruction::Cls);
                assert!(cpu.display.iter().all(|&p| !p));
            }
        }

        mod ret {
            use super::*;

            #[test]
            fn test_ret_restores_pc_from_stack() {
                let mut cpu = new_chip8();
                cpu.stack.push(0x200);
                cpu.execute(Instruction::Ret);
                assert_eq!(cpu.pc, 0x200);
            }

            #[test]
            fn test_ret_pops_stack() {
                let mut cpu = new_chip8();
                cpu.stack.push(0x200);
                cpu.execute(Instruction::Ret);
                assert!(cpu.stack.is_empty());
            }

            #[test]
            #[should_panic(expected = "RET called with empty stack")]
            fn test_ret_empty_stack_panics() {
                let mut cpu = new_chip8();
                cpu.execute(Instruction::Ret);
            }
        }

        mod jp {
            use super::*;

            #[test]
            fn test_jp_sets_pc() {
                let mut cpu = new_chip8();
                cpu.execute(Instruction::Jp { addr: 0xABC });
                assert_eq!(cpu.pc, 0xABC);
            }
        }

        mod call {
            use super::*;

            #[test]
            fn test_call_pushes_pc_to_stack() {
                let mut cpu = new_chip8();
                cpu.pc = 0x200;
                cpu.execute(Instruction::Call { addr: 0x300 });
                assert_eq!(cpu.stack.last().copied(), Some(0x200));
            }

            #[test]
            fn test_call_sets_pc_to_addr() {
                let mut cpu = new_chip8();
                cpu.execute(Instruction::Call { addr: 0x300 });
                assert_eq!(cpu.pc, 0x300);
            }

            #[test]
            fn test_call_and_ret_roundtrip() {
                let mut cpu = new_chip8();
                cpu.pc = 0x200;
                cpu.execute(Instruction::Call { addr: 0x300 });
                assert_eq!(cpu.pc, 0x300);
                cpu.execute(Instruction::Ret);
                assert_eq!(cpu.pc, 0x200);
            }

            #[test]
            #[should_panic(expected = "CALL stack overflow")]
            fn test_call_stack_overflow_panics() {
                let mut cpu = new_chip8();
                for _ in 0..=STACK_SIZE {
                    cpu.execute(Instruction::Call { addr: 0x300 });
                }
            }
        }

        mod se {
            use super::*;

            #[test]
            fn test_se_skips_when_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Se { x: 2, kk: 0x10 });
                assert_eq!(cpu.pc, pc_before + 2);
            }

            #[test]
            fn test_se_does_not_skip_when_not_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Se { x: 2, kk: 0x20 });
                assert_eq!(cpu.pc, pc_before);
            }
        }

        mod sne {
            use super::*;

            #[test]
            fn test_sne_skips_when_not_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Sne { x: 2, kk: 0x20 });
                assert_eq!(cpu.pc, pc_before + 2);
            }

            #[test]
            fn test_sne_does_not_skip_when_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Sne { x: 2, kk: 0x10 });
                assert_eq!(cpu.pc, pc_before);
            }
        }

        mod se_vx_vy {
            use super::*;

            #[test]
            fn test_se_vx_vy_skips_when_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                cpu.v[3] = 0x10;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::SeVxVy { x: 2, y: 3 });
                assert_eq!(cpu.pc, pc_before + 2);
            }

            #[test]
            fn test_se_vx_vy_does_not_skip_when_not_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                cpu.v[3] = 0x20;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::SeVxVy { x: 2, y: 3 });
                assert_eq!(cpu.pc, pc_before);
            }
        }

        mod ld_vx_byte {
            use super::*;

            #[test]
            fn test_ld_vx_byte_sets_register() {
                let mut cpu = new_chip8();
                cpu.execute(Instruction::LdVxByte { x: 3, kk: 0x42 });
                assert_eq!(cpu.v[3], 0x42);
            }
        }

        mod add_vx_byte {
            use super::*;

            #[test]
            fn test_add_vx_byte_adds_value() {
                let mut cpu = new_chip8();
                cpu.v[2] = 10;
                cpu.execute(Instruction::AddVxByte { x: 2, kk: 0x05 });
                assert_eq!(cpu.v[2], 15);
            }

            #[test]
            fn test_add_vx_byte_wraps_on_overflow() {
                let mut cpu = new_chip8();
                cpu.v[0] = 0xFF;
                cpu.execute(Instruction::AddVxByte { x: 0, kk: 0x01 });
                assert_eq!(cpu.v[0], 0x00);
            }
        }

        mod ld_vx_vy {
            use super::*;

            #[test]
            fn test_ld_vx_vy_copies_register() {
                let mut cpu = new_chip8();
                cpu.v[3] = 0x42;
                cpu.execute(Instruction::LdVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0x42);
            }
        }

        mod or_vx_vy {
            use super::*;

            #[test]
            fn test_or_vx_vy() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0b10110000;
                cpu.v[3] = 0b11001100;
                cpu.execute(Instruction::OrVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b11111100);
            }
        }

        mod and_vx_vy {
            use super::*;

            #[test]
            fn test_and_vx_vy() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0b10110000;
                cpu.v[3] = 0b11001100;
                cpu.execute(Instruction::AndVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b10000000);
            }
        }

        mod xor_vx_vy {
            use super::*;

            #[test]
            fn test_xor_vx_vy() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0b10110000;
                cpu.v[3] = 0b11001100;
                cpu.execute(Instruction::XorVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b01111100);
            }
        }

        mod add_vx_vy {
            use super::*;

            #[test]
            fn test_add_vx_vy_no_carry() {
                let mut cpu = new_chip8();
                cpu.v[2] = 10;
                cpu.v[3] = 20;
                cpu.execute(Instruction::AddVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 30);
                assert_eq!(cpu.v[0xF], 0);
            }

            #[test]
            fn test_add_vx_vy_with_carry() {
                let mut cpu = new_chip8();
                cpu.v[2] = 200;
                cpu.v[3] = 100;
                cpu.execute(Instruction::AddVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 44); // 300 - 256 = 44
                assert_eq!(cpu.v[0xF], 1);
            }
        }

        mod sub_vx_vy {
            use super::*;

            #[test]
            fn test_sub_vx_vy_no_borrow() {
                let mut cpu = new_chip8();
                cpu.v[2] = 100;
                cpu.v[3] = 40;
                cpu.execute(Instruction::SubVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 60);
                assert_eq!(cpu.v[0xF], 1); // NOT borrow = 1 because Vx > Vy
            }

            #[test]
            fn test_sub_vx_vy_with_borrow() {
                let mut cpu = new_chip8();
                cpu.v[2] = 40;
                cpu.v[3] = 100;
                cpu.execute(Instruction::SubVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 40u8.wrapping_sub(100)); // wraps around
                assert_eq!(cpu.v[0xF], 0); // NOT borrow = 0 because Vx < Vy
            }
        }

        mod shr_vx {
            use super::*;

            #[test]
            fn test_shr_vx_shifts_vy_into_vx() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[3] = 0b00001010;
                cpu.execute(Instruction::ShrVx { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b00000101);
                assert_eq!(cpu.v[3], 0b00001010); // Vy unchanged
                assert_eq!(cpu.v[0xF], 0);
            }

            #[test]
            fn test_shr_vx_saves_lost_bit() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[3] = 0b00001011;
                cpu.execute(Instruction::ShrVx { x: 2, y: 3 });
                assert_eq!(cpu.v[0xF], 1);
            }

            #[test]
            fn test_shr_vx_vf_is_set_after_result() {
                let mut cpu = new_chip8();
                // x == 0xF: result written to VF, then VF overwritten with flag
                cpu.v[0xF] = 0b00000011;
                cpu.execute(Instruction::ShrVx { x: 0xF, y: 0xF });
                assert_eq!(cpu.v[0xF], 1); // flag (LSB), not shifted result
            }
        }

        mod subn_vx_vy {
            use super::*;

            #[test]
            fn test_subn_vx_vy_no_borrow() {
                let mut cpu = new_chip8();
                cpu.v[2] = 40;
                cpu.v[3] = 100;
                cpu.execute(Instruction::SubnVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 60); // Vy - Vx = 100 - 40
                assert_eq!(cpu.v[0xF], 1); // NOT borrow = 1 because Vy > Vx
            }

            #[test]
            fn test_subn_vx_vy_with_borrow() {
                let mut cpu = new_chip8();
                cpu.v[2] = 100;
                cpu.v[3] = 40;
                cpu.execute(Instruction::SubnVxVy { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 40u8.wrapping_sub(100)); // wraps around
                assert_eq!(cpu.v[0xF], 0); // NOT borrow = 0 because Vy < Vx
            }
        }

        mod shl_vx {
            use super::*;

            #[test]
            fn test_shl_vx_shifts_vy_into_vx() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[3] = 0b00000101;
                cpu.execute(Instruction::ShlVx { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b00001010);
                assert_eq!(cpu.v[3], 0b00000101); // Vy unchanged
                assert_eq!(cpu.v[0xF], 0);
            }

            #[test]
            fn test_shl_vx_saves_lost_bit() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[3] = 0b10000001;
                cpu.execute(Instruction::ShlVx { x: 2, y: 3 });
                assert_eq!(cpu.v[0xF], 1);
            }

            #[test]
            fn test_shl_vx_vf_is_set_after_result() {
                let mut cpu = new_chip8();
                // x == 0xF: result written to VF, then VF overwritten with flag
                cpu.v[0xF] = 0b11000000;
                cpu.execute(Instruction::ShlVx { x: 0xF, y: 0xF });
                assert_eq!(cpu.v[0xF], 1); // flag (MSB), not shifted result
            }
        }

        mod sne_vx_vy {
            use super::*;

            #[test]
            fn test_sne_vx_vy_skips_when_not_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                cpu.v[3] = 0x20;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::SneVxVy { x: 2, y: 3 });
                assert_eq!(cpu.pc, pc_before + 2);
            }

            #[test]
            fn test_sne_vx_vy_does_not_skip_when_equal() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x10;
                cpu.v[3] = 0x10;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::SneVxVy { x: 2, y: 3 });
                assert_eq!(cpu.pc, pc_before);
            }
        }

        mod ld_i {
            use super::*;

            #[test]
            fn test_ld_i_sets_i() {
                let mut cpu = new_chip8();
                cpu.execute(Instruction::LdI { addr: 0x123 });
                assert_eq!(cpu.i, 0x123);
            }
        }

        mod jp_v0 {
            use super::*;

            #[test]
            fn test_jp_v0_jumps_to_addr_plus_v0() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[0] = 0x10;
                cpu.execute(Instruction::JpV0 { addr: 0x200 });
                assert_eq!(cpu.pc, 0x210);
            }

            #[test]
            fn test_jp_v0_with_zero_offset() {
                let mut cpu = new_chip8();
                cpu.v[0] = 0;
                cpu.execute(Instruction::JpV0 { addr: 0x300 });
                assert_eq!(cpu.pc, 0x300);
            }
        }

        mod rnd {
            use super::*;

            #[test]
            fn test_rnd_result_is_masked_by_kk() {
                let mut cpu = new_chip8();
                for _ in 0..100 {
                    cpu.execute(Instruction::Rnd { x: 0, kk: 0x0F });
                    assert!(cpu.v[0] <= 0x0F);
                }
            }

            #[test]
            fn test_rnd_with_zero_mask_is_always_zero() {
                let mut cpu = new_chip8();
                cpu.execute(Instruction::Rnd { x: 0, kk: 0x00 });
                assert_eq!(cpu.v[0], 0);
            }
        }

        mod drw {
            use super::*;

            #[test]
            fn test_drw_turns_on_pixels() {
                let mut cpu = new_chip8();
                // single-row sprite: 0xF0 = 11110000
                cpu.memory[cpu.i as usize] = 0xF0;
                cpu.v[0] = 0; // x = 0
                cpu.v[1] = 0; // y = 0
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert!(cpu.display[0]);
                assert!(cpu.display[1]);
                assert!(cpu.display[2]);
                assert!(cpu.display[3]);
                assert!(!cpu.display[4]);
            }

            #[test]
            fn test_drw_xor_toggles_pixels() {
                let mut cpu = new_chip8();
                cpu.memory[cpu.i as usize] = 0xFF;
                cpu.v[0] = 0;
                cpu.v[1] = 0;
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert!(!cpu.display[0]);
            }

            #[test]
            fn test_drw_sets_vf_on_collision() {
                let mut cpu = new_chip8();
                cpu.memory[cpu.i as usize] = 0xFF;
                cpu.v[0] = 0;
                cpu.v[1] = 0;
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert_eq!(cpu.v[0xF], 0); // first pass, no collision
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert_eq!(cpu.v[0xF], 1); // second pass, collision!
            }

            #[test]
            fn test_drw_wraps_horizontally() {
                let mut cpu = new_chip8();
                cpu.memory[cpu.i as usize] = 0xFF;
                cpu.v[0] = 63; // x near the right edge
                cpu.v[1] = 0;
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert!(cpu.display[63]);
                assert!(cpu.display[0]); // wrapped pixel
            }

            #[test]
            fn test_drw_wraps_vertically() {
                let mut cpu = new_chip8();
                cpu.memory[cpu.i as usize] = 0xFF;
                cpu.memory[cpu.i as usize + 1] = 0xFF;
                cpu.v[0] = 0;
                cpu.v[1] = 31; // y near the bottom edge
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 2 });
                // row at y=31 must be on
                assert!(cpu.display[31 * 64]);
                // the wrapped row at y=0 must be on
                assert!(cpu.display[0]);
            }

            #[test]
            fn test_drw_no_collision_resets_vf() {
                let mut cpu = new_chip8();
                cpu.v[0xF] = 1;
                cpu.memory[cpu.i as usize] = 0xFF;
                cpu.v[0] = 0;
                cpu.v[1] = 0;
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert_eq!(cpu.v[0xF], 0);
            }
        }

        mod skp {
            use super::*;

            #[test]
            fn test_skp_skips_when_key_pressed() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x5;
                cpu.keys[0x5] = true;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Skp { x: 2 });
                assert_eq!(cpu.pc, pc_before + 2);
            }

            #[test]
            fn test_skp_does_not_skip_when_key_not_pressed() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x5;
                cpu.keys[0x5] = false;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Skp { x: 2 });
                assert_eq!(cpu.pc, pc_before);
            }
        }

        mod sknp {
            use super::*;

            #[test]
            fn test_sknp_skips_when_key_not_pressed() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x5;
                cpu.keys[0x5] = false;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Sknp { x: 2 });
                assert_eq!(cpu.pc, pc_before + 2);
            }

            #[test]
            fn test_sknp_does_not_skip_when_key_pressed() {
                let mut cpu = new_chip8();
                cpu.v[2] = 0x5;
                cpu.keys[0x5] = true;
                let pc_before = cpu.pc;
                cpu.execute(Instruction::Sknp { x: 2 });
                assert_eq!(cpu.pc, pc_before);
            }
        }

        mod ld_vx_dt {
            use super::*;

            #[test]
            fn test_ld_vx_dt_reads_timer() {
                let mut cpu = new_chip8();
                cpu.delay_timer = 42;
                cpu.execute(Instruction::LdVxDt { x: 3 });
                assert_eq!(cpu.v[3], 42);
            }
        }

        mod ld_vx_k {
            use super::*;

            #[test]
            fn test_ld_vx_k_stores_key_when_pressed() {
                let mut cpu = new_chip8();
                cpu.keys[0x5] = true;
                cpu.execute(Instruction::LdVxK { x: 2 });
                assert_eq!(cpu.v[2], 0x5);
            }

            #[test]
            fn test_ld_vx_k_rewinds_pc_when_no_key() {
                let mut cpu = new_chip8();
                let pc_before = cpu.pc;
                cpu.execute(Instruction::LdVxK { x: 2 });
                assert_eq!(cpu.pc, pc_before - 2);
            }
        }

        mod ld_dt_vx {
            use super::*;

            #[test]
            fn test_ld_dt_vx_sets_timer() {
                let mut cpu = new_chip8();
                cpu.v[3] = 60;
                cpu.execute(Instruction::LdDtVx { x: 3 });
                assert_eq!(cpu.delay_timer, 60);
            }
        }

        mod ld_st_vx {
            use super::*;

            #[test]
            fn test_ld_st_vx_sets_timer() {
                let mut cpu = new_chip8();
                cpu.v[3] = 30;
                cpu.execute(Instruction::LdStVx { x: 3 });
                assert_eq!(cpu.sound_timer, 30);
            }
        }

        mod add_i_vx {
            use super::*;

            #[test]
            fn test_add_i_vx_adds_to_i() {
                let mut cpu = new_chip8();
                cpu.i = 0x100;
                cpu.v[2] = 0x10;
                cpu.execute(Instruction::AddIVx { x: 2 });
                assert_eq!(cpu.i, 0x110);
            }
        }

        mod ld_f_vx {
            use super::*;

            #[test]
            fn test_ld_f_vx_points_to_font_0() {
                let mut cpu = new_chip8();
                cpu.v[0] = 0x0;
                cpu.execute(Instruction::LdFVx { x: 0 });
                assert_eq!(cpu.i, 0x000);
            }

            #[test]
            fn test_ld_f_vx_points_to_font_1() {
                let mut cpu = new_chip8();
                cpu.v[0] = 0x1;
                cpu.execute(Instruction::LdFVx { x: 0 });
                assert_eq!(cpu.i, 0x005);
            }

            #[test]
            fn test_ld_f_vx_points_to_font_f() {
                let mut cpu = new_chip8();
                cpu.v[0] = 0xF;
                cpu.execute(Instruction::LdFVx { x: 0 });
                assert_eq!(cpu.i, 0x04B);
            }
        }

        mod ld_b_vx {
            use super::*;

            #[test]
            fn test_ld_b_vx_stores_bcd() {
                let mut cpu = new_chip8();
                cpu.i = 0x300;
                cpu.v[2] = 156;
                cpu.execute(Instruction::LdBVx { x: 2 });
                assert_eq!(cpu.memory[0x300], 1); // hundreds
                assert_eq!(cpu.memory[0x301], 5); // tens
                assert_eq!(cpu.memory[0x302], 6); // units
            }

            #[test]
            fn test_ld_b_vx_stores_bcd_small_value() {
                let mut cpu = new_chip8();
                cpu.i = 0x300;
                cpu.v[2] = 7;
                cpu.execute(Instruction::LdBVx { x: 2 });
                assert_eq!(cpu.memory[0x300], 0);
                assert_eq!(cpu.memory[0x301], 0);
                assert_eq!(cpu.memory[0x302], 7);
            }
        }

        mod ld_i_vx {
            use super::*;

            #[test]
            fn test_ld_i_vx_stores_registers() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.i = 0x300;
                cpu.v[0] = 0x11;
                cpu.v[1] = 0x22;
                cpu.v[2] = 0x33;
                cpu.execute(Instruction::LdIVx { x: 2 });
                assert_eq!(cpu.memory[0x300], 0x11);
                assert_eq!(cpu.memory[0x301], 0x22);
                assert_eq!(cpu.memory[0x302], 0x33);
                assert_eq!(cpu.i, 0x303); // I incremented by x + 1 (legacy)
            }
        }

        mod ld_vx_i {
            use super::*;

            #[test]
            fn test_ld_vx_i_loads_registers() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.i = 0x300;
                cpu.memory[0x300] = 0x11;
                cpu.memory[0x301] = 0x22;
                cpu.memory[0x302] = 0x33;
                cpu.execute(Instruction::LdVxI { x: 2 });
                assert_eq!(cpu.v[0], 0x11);
                assert_eq!(cpu.v[1], 0x22);
                assert_eq!(cpu.v[2], 0x33);
                assert_eq!(cpu.i, 0x303); // I incremented by x + 1 (legacy)
            }

            #[test]
            fn test_ld_vx_i_roundtrip_with_ld_i_vx() {
                let mut cpu = new_chip8();
                cpu.i = 0x300;
                cpu.v[0] = 0xAA;
                cpu.v[1] = 0xBB;
                // save — I advances to 0x302
                cpu.execute(Instruction::LdIVx { x: 1 });
                // clear the registers
                cpu.v[0] = 0;
                cpu.v[1] = 0;
                // reset I so the reload reads from the same address
                cpu.i = 0x300;
                // reload — I advances to 0x302 again
                cpu.execute(Instruction::LdVxI { x: 1 });
                assert_eq!(cpu.v[0], 0xAA);
                assert_eq!(cpu.v[1], 0xBB);
            }
        }

        mod unknown {
            use super::*;

            #[test]
            #[should_panic(expected = "Unknown opcode: 0xFFFF")]
            fn test_unknown_opcode_panics() {
                let mut cpu = new_chip8();
                cpu.execute(Instruction::Unknown(0xFFFF));
            }
        }
    }

    mod quirks {
        use super::*;

        mod shift {
            use super::*;

            #[test]
            fn test_shr_modern_shifts_vx_in_place() {
                let mut cpu = new_chip8(); // default = modern
                cpu.v[2] = 0b00001010;
                cpu.v[3] = 0b11111111; // Vy value is ignored
                cpu.execute(Instruction::ShrVx { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b00000101); // Vx shifted in-place
                assert_eq!(cpu.v[3], 0b11111111); // Vy unchanged
                assert_eq!(cpu.v[0xF], 0); // LSB of Vx was 0
            }

            #[test]
            fn test_shr_legacy_shifts_vy_into_vx() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[2] = 0b00000000; // Vx initial value doesn't matter
                cpu.v[3] = 0b00001010;
                cpu.execute(Instruction::ShrVx { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b00000101); // result from Vy
                assert_eq!(cpu.v[3], 0b00001010); // Vy unchanged
                assert_eq!(cpu.v[0xF], 0); // LSB of Vy was 0
            }

            #[test]
            fn test_shl_modern_shifts_vx_in_place() {
                let mut cpu = new_chip8(); // default = modern
                cpu.v[2] = 0b00000101;
                cpu.v[3] = 0b11111111; // Vy value is ignored
                cpu.execute(Instruction::ShlVx { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b00001010); // Vx shifted in-place
                assert_eq!(cpu.v[3], 0b11111111); // Vy unchanged
                assert_eq!(cpu.v[0xF], 0); // MSB of Vx was 0
            }

            #[test]
            fn test_shl_legacy_shifts_vy_into_vx() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[2] = 0b00000000; // Vx initial value doesn't matter
                cpu.v[3] = 0b00000101;
                cpu.execute(Instruction::ShlVx { x: 2, y: 3 });
                assert_eq!(cpu.v[2], 0b00001010); // result from Vy
                assert_eq!(cpu.v[3], 0b00000101); // Vy unchanged
                assert_eq!(cpu.v[0xF], 0); // MSB of Vy was 0
            }
        }

        mod load_store {
            use super::*;

            #[test]
            fn test_ld_i_vx_modern_leaves_i_unchanged() {
                let mut cpu = new_chip8(); // default = modern
                cpu.i = 0x300;
                cpu.v[0] = 0xAA;
                cpu.v[1] = 0xBB;
                cpu.execute(Instruction::LdIVx { x: 1 });
                assert_eq!(cpu.memory[0x300], 0xAA);
                assert_eq!(cpu.memory[0x301], 0xBB);
                assert_eq!(cpu.i, 0x300); // I unchanged
            }

            #[test]
            fn test_ld_i_vx_legacy_increments_i() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.i = 0x300;
                cpu.v[0] = 0xAA;
                cpu.v[1] = 0xBB;
                cpu.execute(Instruction::LdIVx { x: 1 });
                assert_eq!(cpu.memory[0x300], 0xAA);
                assert_eq!(cpu.memory[0x301], 0xBB);
                assert_eq!(cpu.i, 0x302); // I += x+1 = 2
            }

            #[test]
            fn test_ld_vx_i_modern_leaves_i_unchanged() {
                let mut cpu = new_chip8(); // default = modern
                cpu.i = 0x300;
                cpu.memory[0x300] = 0xAA;
                cpu.memory[0x301] = 0xBB;
                cpu.execute(Instruction::LdVxI { x: 1 });
                assert_eq!(cpu.v[0], 0xAA);
                assert_eq!(cpu.v[1], 0xBB);
                assert_eq!(cpu.i, 0x300); // I unchanged
            }

            #[test]
            fn test_ld_vx_i_legacy_increments_i() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.i = 0x300;
                cpu.memory[0x300] = 0xAA;
                cpu.memory[0x301] = 0xBB;
                cpu.execute(Instruction::LdVxI { x: 1 });
                assert_eq!(cpu.v[0], 0xAA);
                assert_eq!(cpu.v[1], 0xBB);
                assert_eq!(cpu.i, 0x302); // I += x+1 = 2
            }
        }

        mod jump {
            use super::*;

            #[test]
            fn test_jp_v0_modern_uses_vx_as_offset() {
                let mut cpu = new_chip8(); // default = modern
                // addr = 0x200, upper nibble x = 2; V[0] differs to prove it's ignored
                cpu.v[0] = 0xFF;
                cpu.v[2] = 0x30;
                cpu.execute(Instruction::JpV0 { addr: 0x200 });
                assert_eq!(cpu.pc, 0x200 + 0x30); // addr + V[2], not V[0]
            }

            #[test]
            fn test_jp_v0_legacy_uses_v0_as_offset() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[0] = 0x30;
                cpu.v[2] = 0xFF; // V[2] is ignored in legacy mode
                cpu.execute(Instruction::JpV0 { addr: 0x200 });
                assert_eq!(cpu.pc, 0x200 + 0x30); // addr + V[0]
            }
        }

        mod clip_sprites {
            use super::*;

            #[test]
            fn test_drw_modern_wraps_past_right_edge() {
                let mut cpu = new_chip8(); // default = modern (wrap)
                cpu.memory[cpu.i as usize] = 0xFF; // 8 pixels wide
                cpu.v[0] = 62; // x=62, pixels at 62,63 and then 0,1,2,3,4,5
                cpu.v[1] = 0;
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert!(cpu.display[62]); // pixel drawn
                assert!(cpu.display[63]); // pixel drawn
                assert!(cpu.display[0]); // wrapped pixel drawn
            }

            #[test]
            fn test_drw_legacy_clips_past_right_edge() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.memory[cpu.i as usize] = 0xFF; // 8 pixels wide
                cpu.v[0] = 62; // x=62, only pixels at 62 and 63 are in bounds
                cpu.v[1] = 0;
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 1 });
                assert!(cpu.display[62]); // pixel drawn
                assert!(cpu.display[63]); // pixel drawn
                assert!(!cpu.display[0]); // NOT drawn (clipped)
            }

            #[test]
            fn test_drw_modern_wraps_past_bottom_edge() {
                let mut cpu = new_chip8(); // default = modern (wrap)
                cpu.memory[cpu.i as usize] = 0xFF;
                cpu.memory[cpu.i as usize + 1] = 0xFF;
                cpu.v[0] = 0;
                cpu.v[1] = 31; // y=31, second row wraps to y=0
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 2 });
                assert!(cpu.display[31 * DISPLAY_WIDTH]); // row at y=31 drawn
                assert!(cpu.display[0]); // wrapped row at y=0 drawn
            }

            #[test]
            fn test_drw_legacy_clips_past_bottom_edge() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.memory[cpu.i as usize] = 0xFF;
                cpu.memory[cpu.i as usize + 1] = 0xFF;
                cpu.v[0] = 0;
                cpu.v[1] = 31; // y=31, second row is out of bounds
                cpu.execute(Instruction::Drw { x: 0, y: 1, n: 2 });
                assert!(cpu.display[31 * DISPLAY_WIDTH]); // row at y=31 drawn
                assert!(!cpu.display[0]); // NOT drawn (clipped)
            }
        }

        mod reset_vf_after_logical {
            use super::*;

            #[test]
            fn test_or_modern_leaves_vf_unchanged() {
                let mut cpu = new_chip8(); // default = modern
                cpu.v[0xF] = 0x42; // set VF to a known value
                cpu.v[0] = 0xF0;
                cpu.v[1] = 0x0F;
                cpu.execute(Instruction::OrVxVy { x: 0, y: 1 });
                assert_eq!(cpu.v[0], 0xFF);
                assert_eq!(cpu.v[0xF], 0x42); // VF unchanged
            }

            #[test]
            fn test_or_legacy_resets_vf_to_zero() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[0xF] = 0x42;
                cpu.v[0] = 0xF0;
                cpu.v[1] = 0x0F;
                cpu.execute(Instruction::OrVxVy { x: 0, y: 1 });
                assert_eq!(cpu.v[0], 0xFF);
                assert_eq!(cpu.v[0xF], 0); // VF reset to 0
            }

            #[test]
            fn test_and_modern_leaves_vf_unchanged() {
                let mut cpu = new_chip8();
                cpu.v[0xF] = 0x42;
                cpu.v[0] = 0xFF;
                cpu.v[1] = 0x0F;
                cpu.execute(Instruction::AndVxVy { x: 0, y: 1 });
                assert_eq!(cpu.v[0], 0x0F);
                assert_eq!(cpu.v[0xF], 0x42);
            }

            #[test]
            fn test_and_legacy_resets_vf_to_zero() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[0xF] = 0x42;
                cpu.v[0] = 0xFF;
                cpu.v[1] = 0x0F;
                cpu.execute(Instruction::AndVxVy { x: 0, y: 1 });
                assert_eq!(cpu.v[0], 0x0F);
                assert_eq!(cpu.v[0xF], 0);
            }

            #[test]
            fn test_xor_modern_leaves_vf_unchanged() {
                let mut cpu = new_chip8();
                cpu.v[0xF] = 0x42;
                cpu.v[0] = 0xFF;
                cpu.v[1] = 0x0F;
                cpu.execute(Instruction::XorVxVy { x: 0, y: 1 });
                assert_eq!(cpu.v[0], 0xF0);
                assert_eq!(cpu.v[0xF], 0x42);
            }

            #[test]
            fn test_xor_legacy_resets_vf_to_zero() {
                let mut cpu = Chip8::new(Chip8Config::legacy());
                cpu.v[0xF] = 0x42;
                cpu.v[0] = 0xFF;
                cpu.v[1] = 0x0F;
                cpu.execute(Instruction::XorVxVy { x: 0, y: 1 });
                assert_eq!(cpu.v[0], 0xF0);
                assert_eq!(cpu.v[0xF], 0);
            }
        }
    }

    mod tick_timers {
        use super::*;

        #[test]
        fn test_tick_decrements_delay_timer() {
            let mut cpu = new_chip8();
            cpu.delay_timer = 10;
            cpu.tick_timers();
            assert_eq!(cpu.delay_timer, 9);
        }

        #[test]
        fn test_tick_decrements_sound_timer() {
            let mut cpu = new_chip8();
            cpu.sound_timer = 5;
            cpu.tick_timers();
            assert_eq!(cpu.sound_timer, 4);
        }

        #[test]
        fn test_tick_does_not_underflow_delay_timer() {
            let mut cpu = new_chip8();
            cpu.delay_timer = 0;
            cpu.tick_timers();
            assert_eq!(cpu.delay_timer, 0);
        }

        #[test]
        fn test_tick_does_not_underflow_sound_timer() {
            let mut cpu = new_chip8();
            cpu.sound_timer = 0;
            cpu.tick_timers();
            assert_eq!(cpu.sound_timer, 0);
        }

        #[test]
        fn test_is_beeping_when_sound_timer_nonzero() {
            let mut cpu = new_chip8();
            cpu.sound_timer = 1;
            assert!(cpu.is_beeping());
            cpu.tick_timers();
            assert!(!cpu.is_beeping());
        }
    }
}
