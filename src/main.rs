mod disasm;
mod ui;
mod vm;

fn trace(s: &str) {
    println!("{s}");
}

fn main() {
    let mut chip = vm::Chip8::new();

    // draw_frame_buffer(&chip);

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--disasm" {
        disasm::chip8_disasm_main(&args[2]);
        return;
    }
    if args.len() != 2 {
        eprintln!("Expect 1 argument: a rom filename to run");
        std::process::exit(1);
    }

    let rom: Vec<u8> = std::fs::read(&args[1]).expect("failed to read rom file");
    if rom.len() % 2 == 1 {
        panic!("The rom.len() ({}) is not even", rom.len());
    }

    chip.load_rom(&rom);

    loop {
        chip.process_instruction();
    }
}
