mod chip8;
use chip8::{Chip8, Chip8Config};

use minifb::{Key, Window, WindowOptions};

const SCALE: usize = 10;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const WINDOW_WIDTH: usize = DISPLAY_WIDTH * SCALE;
const WINDOW_HEIGHT: usize = DISPLAY_HEIGHT * SCALE;
const CYCLES_PER_FRAME: u32 = 10;

const KEYMAP: [(Key, usize); 16] = [
    (Key::X,    0x0),
    (Key::Key1, 0x1),
    (Key::Key2, 0x2),
    (Key::Key3, 0x3),
    (Key::Q,    0x4),
    (Key::W,    0x5),
    (Key::E,    0x6),
    (Key::A,    0x7),
    (Key::S,    0x8),
    (Key::D,    0x9),
    (Key::Z,    0xA),
    (Key::C,    0xB),
    (Key::Key4, 0xC),
    (Key::R,    0xD),
    (Key::F,    0xE),
    (Key::V,    0xF),
];

fn main() {
    let rom_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: chip8-emu <rom>");
        std::process::exit(1);
    });

    let rom = std::fs::read(&rom_path).unwrap_or_else(|e| {
        eprintln!("Failed to read ROM '{}': {}", rom_path, e);
        std::process::exit(1);
    });

    let mut cpu = Chip8::new(Chip8Config::default());
    cpu.load_rom(&rom).unwrap_or_else(|e| {
        eprintln!("Failed to load ROM: {}", e);
        std::process::exit(1);
    });

    let mut window = Window::new("CHIP-8", WINDOW_WIDTH, WINDOW_HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            eprintln!("Failed to create window: {}", e);
            std::process::exit(1);
        });

    window.set_target_fps(60);

    let mut buffer = vec![0u32; WINDOW_WIDTH * WINDOW_HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (key, chip8_key) in &KEYMAP {
            cpu.set_key(*chip8_key, window.is_key_down(*key));
        }

        for _ in 0..CYCLES_PER_FRAME {
            let opcode = cpu.fetch();
            let instruction = cpu.decode(opcode);
            cpu.execute(instruction);
        }

        cpu.tick_timers();

        let display = cpu.display();
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                let color = if display[row * DISPLAY_WIDTH + col] { 0xFFFFFF } else { 0x000000 };
                for dy in 0..SCALE {
                    for dx in 0..SCALE {
                        buffer[(row * SCALE + dy) * WINDOW_WIDTH + col * SCALE + dx] = color;
                    }
                }
            }
        }

        window.update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();
    }
}
