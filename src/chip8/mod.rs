//! CHIP-8 Emulator
//!
//! Technical reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
//! Guide: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
//! Test suite: https://github.com/Timendus/chip8-test-suite
//!

mod config;
mod constants;
mod cpu;
mod instruction;

pub use config::Chip8Config;
pub use cpu::Chip8;
pub use instruction::Instruction;
