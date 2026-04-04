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
