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
