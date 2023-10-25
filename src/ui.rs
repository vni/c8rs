/*
use crossterm::{
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};
*/

use crate::Chip8;
// use crossterm::QueueableCommand;
use std::io::Write;

pub struct Terminal;

impl Terminal {
    pub fn new() {
        crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen).unwrap();
        crossterm::terminal::enable_raw_mode().expect("failed to enter raw mode");
    }

    pub fn print_state(chip: &Chip8) {
        crossterm::cursor::MoveTo(0, 0);
        println!(
            ", PC: {:04x}, SP: {:04x}, I: {:04x}",
            chip.pc, chip.sp, chip.index
        );
        print!("REGS:");
        for r in chip.regs {
            print!(" {:04x}", r);
        }
    }

    pub fn draw_frame_buffer(chip: &Chip8) {
        let mut stdout = std::io::stdout().lock();
        crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 4)).unwrap();
        for row in 0..32 {
            for col in (0..64).rev() {
                if chip.frame_buffer[row] & (1 << col) != 0 {
                    crossterm::execute!(
                        stdout,
                        crossterm::style::SetBackgroundColor(crossterm::style::Color::Yellow)
                    )
                    .unwrap();
                    write!(stdout, "  ").unwrap();
                    crossterm::execute!(std::io::stdout(), crossterm::style::ResetColor).unwrap();
                } else {
                    print!("  ");
                }
            }
            println!();
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().expect("failed to disable raw mode");
    }
}

// use minifb::{Key, Scale, Window, WindowOptions};
// const CELL_SIZE: usize = 10;
// const WINDOW_WIDTH: usize = 64 /* * CELL_SIZE */;
// const WINDOW_HEIGHT: usize = 32 /* * CELL_SIZE */;
// fn init_fbwindow() -> Window {
//     let mut window = Window::new(
//         "chip8 emulator",
//         WINDOW_WIDTH,
//         WINDOW_HEIGHT,
//         WindowOptions {
//             resize: false,
//             scale: Scale::X16,
//             ..WindowOptions::default()
//         },
//     )
//     .expect("Unable to create window");
//     window.limit_update_rate(Some(std::time::Duration::from_millis(16_600)));

//     // let mub buffer: Vec<u32> = Vec::with_capacity(WINDOW_WIDTH * WINDOW_HEIGHT);
//     window
// }

// let mut window = init_fbwindow();
// let window_buffer: [u32; WINDOW_WIDTH * WINDOW_HEIGHT] =
//     [0x000088AA_u32; WINDOW_WIDTH * WINDOW_HEIGHT];
// while window.is_open() && !window.is_key_down(Key::Escape) {
//     println!("window.is_open loop");
//     //
//     window
//         .update_with_buffer(&window_buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
//         .expect("failed to update the window");
// }
