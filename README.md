```
 ██████╗██╗  ██╗██╗██████╗        █████╗
██╔════╝██║  ██║██║██╔══██╗      ██╔══██╗
██║     ███████║██║██████╔╝█████╗╚█████╔╝
██║     ██╔══██║██║██╔═══╝ ╚════╝██╔══██╗
╚██████╗██║  ██║██║██║           ╚█████╔╝
 ╚═════╝╚═╝  ╚═╝╚═╝╚═╝            ╚════╝
```

# chip8-emu

A CHIP-8 interpreter written in Rust, faithfully emulating the original 1977 COSMAC VIP behavior by default, with optional modern (CHIP-48/SCHIP) quirks.

---

## Features

- Full CHIP-8 instruction set — all 35 opcodes
- Configurable quirks covering every known behavioral difference between the 1977 COSMAC VIP and modern interpreters
- 640×320 display (64×32 scaled 10×) via [minifb](https://github.com/emoon/rust_minifb)
- 60 Hz game loop with 10 CPU cycles per frame
- Keyboard input mapped to the original 16-key hex keypad

---

## Building

Requires Rust 1.75+ and a C compiler (for minifb's thin native layer).

```sh
cargo build --release
```

---

## Running

```sh
cargo run --release -- path/to/rom.ch8
```

Press `Escape` or close the window to exit.

---

## Keyboard Layout

The original CHIP-8 keypad was a 4×4 hex grid. It maps to the left side of a standard keyboard:

```
CHIP-8 Keypad        Keyboard
+-+-+-+-+            +-+-+-+-+
|1|2|3|C|            |1|2|3|4|
+-+-+-+-+            +-+-+-+-+
|4|5|6|D|    -->     |Q|W|E|R|
+-+-+-+-+            +-+-+-+-+
|7|8|9|E|            |A|S|D|F|
+-+-+-+-+            +-+-+-+-+
|A|0|B|F|            |Z|X|C|V|
+-+-+-+-+            +-+-+-+-+
```

---

## Quirks

CHIP-8 has a long history of subtly incompatible interpreters. This emulator defaults to the original 1977 COSMAC VIP behavior. Quirks can be toggled through `Chip8Config`:

| Flag | Legacy (default) | Modern (`Chip8Config::modern()`) |
|---|---|---|
| `shift_uses_vy` | `SHR`/`SHL` read from Vy, store into Vx | Shift Vx in-place, ignore Vy |
| `load_store_increments_i` | `FX55`/`FX65` advance I by X+1 | I is left unchanged |
| `jump_uses_v0` | `BNNN` jumps to NNN + V0 | `BXNN` jumps to XNN + VX |
| `clip_sprites` | Sprites clipped at screen edges | Sprites wrap around |
| `reset_vf_after_logical` | VF reset to 0 after OR/AND/XOR | VF unchanged |

---

## Project Structure

```
src/
├── main.rs              # Window, game loop, keyboard input
└── chip8/
    ├── mod.rs           # Public re-exports
    ├── constants.rs     # Memory layout, display dimensions, fontset
    ├── instruction.rs   # Instruction enum (all 35 opcodes)
    ├── config.rs        # Chip8Config — legacy and modern presets
    └── cpu.rs           # Chip8 struct, fetch/decode/execute, timers
```

---

## References

- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Tobias V. Langhoff — Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [Timendus CHIP-8 Test Suite](https://github.com/Timendus/chip8-test-suite)

---
