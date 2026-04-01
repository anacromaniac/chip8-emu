# Cowgod's Chip-8 Technical Reference v1.0

> Original source: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM  
> Author: Thomas P. Greene  
> Date: August 30, 1997

---

## 0.0 - Table of Contents

- [1.0 - About Chip-8](#10---about-chip-8)
- [2.0 - Chip-8 Specifications](#20---chip-8-specifications)
  - [2.1 - Memory](#21---memory)
  - [2.2 - Registers](#22---registers)
  - [2.3 - Keyboard](#23---keyboard)
  - [2.4 - Display](#24---display)
  - [2.5 - Timers & Sound](#25---timers--sound)
- [3.0 - Chip-8 Instructions](#30---chip-8-instructions)
  - [3.1 - Standard Instructions](#31---standard-chip-8-instructions)
  - [3.2 - Super Chip-48 Instructions](#32---super-chip-48-instructions)
- [4.0 - Interpreters](#40---interpreters)
- [5.0 - Credits](#50---credits)

---

## 1.0 - About Chip-8

Chip-8 is a simple, interpreted, programming language which was first used on some
do-it-yourself computer systems in the late 1970s and early 1980s. The COSMAC VIP,
DREAM 6800, and ETI 660 computers are a few examples. These computers typically
were designed to use a television as a display, had between 1 and 4K of RAM, and
used a 16-key hexadecimal keypad for input. The interpreter took up only 512 bytes
of memory, and programs were even smaller.

In the early 1990s, the Chip-8 language was revived by Andreas Gustafsson, who
created a Chip-8 interpreter for the HP48 graphing calculator called Chip-48.
Chip-48 later begat Super Chip-48, which allowed higher resolution graphics.

---

## 2.0 - Chip-8 Specifications

### 2.1 - Memory

The Chip-8 language is capable of accessing up to 4KB (4,096 bytes) of RAM, from
location `0x000` (0) to `0xFFF` (4095). The first 512 bytes, from `0x000` to
`0x1FF`, are where the original interpreter was located, and should not be used
by programs.

Most Chip-8 programs start at location `0x200` (512), but some begin at `0x600`
(1536). Programs beginning at `0x600` are intended for the ETI 660 computer.

**Memory Map:**

```
+---------------+= 0xFFF (4095) End of Chip-8 RAM
|               |
| 0x200 to 0xFFF|
|    Chip-8     |
| Program / Data|
|    Space      |
|               |
+- - - - - - - -+= 0x600 (1536) Start of ETI 660 Chip-8 programs
|               |
+---------------+= 0x200 (512) Start of most Chip-8 programs
| 0x000 to 0x1FF|
| Reserved for  |
|  interpreter  |
+---------------+= 0x000 (0) Start of Chip-8 RAM
```

---

### 2.2 - Registers

Chip-8 has **16 general purpose 8-bit registers**, referred to as `Vx` where `x`
is a hexadecimal digit (0 through F).

- **`I`** — 16-bit register, generally used to store memory addresses (only lowest 12 bits used)
- **`VF`** — should not be used by programs; used as a flag by some instructions
- **`DT`** — delay timer, 8-bit, decremented at 60Hz when non-zero
- **`ST`** — sound timer, 8-bit, decremented at 60Hz when non-zero; buzzer sounds when > 0
- **`PC`** — program counter, 16-bit, stores currently executing address (pseudo-register)
- **`SP`** — stack pointer, 8-bit, points to topmost level of the stack (pseudo-register)
- **Stack** — array of 16 x 16-bit values; allows up to 16 levels of nested subroutines

---

### 2.3 - Keyboard

The original Chip-8 computers had a 16-key hexadecimal keypad:

```
+---+---+---+---+
| 1 | 2 | 3 | C |
+---+---+---+---+
| 4 | 5 | 6 | D |
+---+---+---+---+
| 7 | 8 | 9 | E |
+---+---+---+---+
| A | 0 | B | F |
+---+---+---+---+
```

---

### 2.4 - Display

The original Chip-8 display is **64x32 pixels, monochrome**.

```
(0,0)               (63,0)
+-------------------+
|                   |
+-------------------+
(0,31)             (63,31)
```

Chip-8 draws graphics through **sprites** — groups of bytes as binary representations
of pictures. Sprites may be up to 15 bytes (8x15 pixels).

Programs may refer to built-in sprites for hexadecimal digits 0–F. These are 5 bytes
long (8x5 pixels) and stored in the interpreter area (`0x000` to `0x1FF`).

**Built-in Font (hex digits 0–F):**

```
"0": 0xF0, 0x90, 0x90, 0x90, 0xF0
"1": 0x20, 0x60, 0x20, 0x20, 0x70
"2": 0xF0, 0x10, 0xF0, 0x80, 0xF0
"3": 0xF0, 0x10, 0xF0, 0x10, 0xF0
"4": 0x90, 0x90, 0xF0, 0x10, 0x10
"5": 0xF0, 0x80, 0xF0, 0x10, 0xF0
"6": 0xF0, 0x80, 0xF0, 0x90, 0xF0
"7": 0xF0, 0x10, 0x20, 0x40, 0x40
"8": 0xF0, 0x90, 0xF0, 0x90, 0xF0
"9": 0xF0, 0x90, 0xF0, 0x10, 0xF0
"A": 0xF0, 0x90, 0xF0, 0x90, 0x90
"B": 0xE0, 0x90, 0xE0, 0x90, 0xE0
"C": 0xF0, 0x80, 0x80, 0x80, 0xF0
"D": 0xE0, 0x90, 0x90, 0x90, 0xE0
"E": 0xF0, 0x80, 0xF0, 0x80, 0xF0
"F": 0xF0, 0x80, 0xF0, 0x80, 0x80
```

---

### 2.5 - Timers & Sound

Chip-8 provides 2 timers:

- **Delay Timer (DT)** — decrements at 60Hz when non-zero; used for timing events
- **Sound Timer (ST)** — decrements at 60Hz when non-zero; buzzer sounds while ST > 0

The sound has only one tone; its frequency is decided by the interpreter author.

---

## 3.0 - Chip-8 Instructions

The original Chip-8 includes **36 instructions**. Super Chip-48 added 10 more (total 46).

All instructions are **2 bytes long**, stored most-significant-byte first.

**Variable notation:**

| Symbol | Meaning |
|--------|---------|
| `nnn` or `addr` | 12-bit value, lowest 12 bits of the instruction |
| `n` or `nibble` | 4-bit value, lowest 4 bits of the instruction |
| `x` | 4-bit value, lower 4 bits of the high byte |
| `y` | 4-bit value, upper 4 bits of the low byte |
| `kk` or `byte` | 8-bit value, lowest 8 bits of the instruction |

---

### 3.1 - Standard Chip-8 Instructions

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| `0nnn` | SYS addr | Jump to machine code routine at nnn. Ignored by modern interpreters. |
| `00E0` | CLS | Clear the display. |
| `00EE` | RET | Return from subroutine. PC = top of stack, SP -= 1. |
| `1nnn` | JP addr | Jump to location nnn. PC = nnn. |
| `2nnn` | CALL addr | Call subroutine at nnn. SP += 1, stack[SP] = PC, PC = nnn. |
| `3xkk` | SE Vx, byte | Skip next instruction if Vx == kk. |
| `4xkk` | SNE Vx, byte | Skip next instruction if Vx != kk. |
| `5xy0` | SE Vx, Vy | Skip next instruction if Vx == Vy. |
| `6xkk` | LD Vx, byte | Set Vx = kk. |
| `7xkk` | ADD Vx, byte | Set Vx = Vx + kk. |
| `8xy0` | LD Vx, Vy | Set Vx = Vy. |
| `8xy1` | OR Vx, Vy | Set Vx = Vx OR Vy. |
| `8xy2` | AND Vx, Vy | Set Vx = Vx AND Vy. |
| `8xy3` | XOR Vx, Vy | Set Vx = Vx XOR Vy. |
| `8xy4` | ADD Vx, Vy | Set Vx = Vx + Vy, VF = carry. |
| `8xy5` | SUB Vx, Vy | Set Vx = Vx - Vy, VF = NOT borrow. |
| `8xy6` | SHR Vx | Set Vx = Vx SHR 1. VF = least-significant bit before shift. |
| `8xy7` | SUBN Vx, Vy | Set Vx = Vy - Vx, VF = NOT borrow. |
| `8xyE` | SHL Vx | Set Vx = Vx SHL 1. VF = most-significant bit before shift. |
| `9xy0` | SNE Vx, Vy | Skip next instruction if Vx != Vy. |
| `Annn` | LD I, addr | Set I = nnn. |
| `Bnnn` | JP V0, addr | Jump to location nnn + V0. |
| `Cxkk` | RND Vx, byte | Set Vx = random byte AND kk. |
| `Dxyn` | DRW Vx, Vy, n | Display n-byte sprite at (Vx, Vy). VF = collision. Sprites XORed onto screen, wrap around edges. |
| `Ex9E` | SKP Vx | Skip next instruction if key with value Vx is pressed. |
| `ExA1` | SKNP Vx | Skip next instruction if key with value Vx is not pressed. |
| `Fx07` | LD Vx, DT | Set Vx = delay timer value. |
| `Fx0A` | LD Vx, K | Wait for key press, store key value in Vx. All execution stops. |
| `Fx15` | LD DT, Vx | Set delay timer = Vx. |
| `Fx18` | LD ST, Vx | Set sound timer = Vx. |
| `Fx1E` | ADD I, Vx | Set I = I + Vx. |
| `Fx29` | LD F, Vx | Set I = location of sprite for digit Vx. |
| `Fx33` | LD B, Vx | Store BCD representation of Vx in memory at I, I+1, I+2. |
| `Fx55` | LD [I], Vx | Store registers V0 through Vx in memory starting at I. |
| `Fx65` | LD Vx, [I] | Read registers V0 through Vx from memory starting at I. |

---

### 3.2 - Super Chip-48 Instructions

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| `00Cn` | SCD nibble | Scroll display N lines down. |
| `00FB` | SCR | Scroll display 4 pixels right. |
| `00FC` | SCL | Scroll display 4 pixels left. |
| `00FD` | EXIT | Exit the interpreter. |
| `00FE` | LOW | Disable extended screen mode. |
| `00FF` | HIGH | Enable extended screen mode (128x64). |
| `Dxy0` | DRW Vx, Vy, 0 | Draw 16x16 sprite (extended mode). |
| `Fx30` | LD HF, Vx | Set I = location of large sprite for digit Vx. |
| `Fx75` | LD R, Vx | Store V0–Vx in RPL user flags. |
| `Fx85` | LD Vx, R | Read V0–Vx from RPL user flags. |

---

## 4.0 - Interpreters

| Title | Version | Author | Platform |
|-------|---------|--------|----------|
| Chip-48 | 2.20 | Andreas Gustafsson | HP48 |
| Chip8 | 1.1 | Paul Robson | DOS |
| Chip-8 Emulator | 2.0.0 | David Winter | DOS |
| CowChip | 0.1 | Thomas P. Greene | Windows 3.1 |
| DREAM MON | 1.1 | Paul Hayter | Amiga |
| Super Chip-48 | 1.1 | Erik Bryntse | HP48 |
| Vision-8 | 1.0 | Marcel de Kogel | DOS, Adam, MSX, ColecoVision |

---

## 5.0 - Credits

Document compiled by Thomas P. Greene (cowgod@rockpile.com).

Sources: personal research, correspondence with David Winter, David Winter's emulator
documentation, Christian Egeberg's Chipper documentation, Marcel de Kogel's Vision-8
source code, Paul Hayter's DREAM MON documentation, Paul Robson's web page, and
Andreas Gustafsson's Chip-48 documentation.
