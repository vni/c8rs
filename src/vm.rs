use rand::Rng;
use std::io::Read;
use std::io::Write;
// use ui::Terminal;

// start of the VideoMemory (256 bytes)
const VIDEO_MEMORY_OFFSET: usize = 0xF00;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_WIDTH: usize = 64;
const DISPLAY_WIDTH_IN_BYTES: usize = DISPLAY_WIDTH / 8;

// chip8 font, 4x5
const CHIP8_FONTSET: [u8; 80] = [
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

#[rustfmt::skip]
static KEYBOARD: [bool; 16] = [
    false, false, false, false,
    false, false, false, false,
    false, false, false, false,
    false, false, false, false,
];

#[derive(PartialEq, Debug)]
pub struct Chip8 {
    regs: [u8; 16],
    index: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16], //??32 // TODO: should be part of the memory // 0xEA0
    sound_timer: u8,  // buzzes while activated. Decreases at 60Hz. When 0 -> deactivates.
    delay_timer: u8,  // decreases till 0 at rate of 60Hz. When it 0 -> stops.
    // frame_buffer: [u64; 32], // TODO: Should be part of the memory
    memory: [u8; 4096],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            regs: [0u8; 16],
            index: 0,
            pc: 0x200,
            sp: 0,
            stack: [0u16; 16], // TODO: move it to memory[]
            sound_timer: 0,
            delay_timer: 0,
            // frame_buffer: [0u64; 32], // TODO: move it to memory[]
            memory: [0u8; 4096],
        };

        // setup 'bios': set font to the memory addresses 0 .. 0x200
        // println!("set font at chip.memory[0..]");
        for (i, v) in CHIP8_FONTSET.iter().enumerate() {
            chip8.memory[i] = *v;
        }

        chip8
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        for (i, v) in rom.iter().enumerate() {
            self.memory[0x200 + i] = *v;
        }
    }

    /// auxiliary function used for debug purposes
    /// prints the first 512 (0x200) bytes of memory
    /// which are expected to zeroed and populated with font
    #[allow(dead_code)]
    fn print_bios(&self) {
        println!("first 512 (0x200) bytes of memory:");
        for i in 0..512 {
            if (i % 8) == 0 {
                print!("0x{:06x}:", i);
            }
            print!(" {:02x}", self.memory[i]);
            if ((i + 1) % 8) == 0 {
                println!();
            }
        }
    }

    fn print_state(&self) {
        print!("[");
        for i in 0..16 {
            print!(" {:02x}", self.regs[i]);
        }
        print!(
            " ] PC: {:04x}, SP: {:04x}, I: {:04x}",
            self.pc, self.sp, self.index
        );
        println!();
    }

    pub fn process_instruction(&mut self) {
        self.print_state();

        let op: u16 = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;
        print!("{:04x}: {:04x} ", self.pc, op);
        crate::disasm::disasm_inst(op);
        self.pc += 2;

        // ui::Terminal::print_state(&self, &op);

        let a: u8 = ((op & 0xf000) >> 12) as u8;
        let b: u8 = ((op & 0x0f00) >> 8) as u8;
        let c: u8 = ((op & 0x00f0) >> 4) as u8;
        let d: u8 = (op & 0x000f) as u8;

        match (a, b, c, d) {
            // CLS: clear the display
            (0, 0, 0xE, 0) => {
                for i in 0xF00..=0xFFF {
                    self.memory[i] = 0u8;
                }
            }
            // RET: return
            (0, 0, 0xE, 0xE) => {
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1; // FIXME: check for underflow
            }
            // SYS addr;  obsolete
            (0, _, _, _) => unimplemented!("`SYS addr` instruction is obsolete"),
            //jump NNN,
            (1, a, b, c) => {
                self.pc = addr(a, b, c);
            }
            //call NNN
            (2, a, b, c) => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = addr(a, b, c);
            }
            //SE Vx, byte; if vx == kk => skip next instruction
            (3, x, a, b) => {
                if self.regs[x as usize] == byte(a, b) {
                    self.pc += 2;
                }
            }
            //SNE Vx, byte; if vx != kk => skip next instruction
            (4, x, a, b) => {
                if self.regs[x as usize] != byte(a, b) {
                    self.pc += 2;
                }
            }
            //SE Vx, Vy;  0x5XY0 => if vx == vy => skip next instruction
            (5, x, y, 0) => {
                if self.regs[x as usize] == self.regs[y as usize] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte;  0x6XNN => vx := NN, //mov imm
            (6, x, a, b) => {
                self.regs[x as usize] = byte(a, b);
            }
            // ADD Vx, byte;  0x7XNN => vx += NN, //ADD Vx, byte
            (7, x, a, b) => {
                self.regs[x as usize] = self.regs[x as usize].wrapping_add(byte(a, b));
            }
            // LD Vx, Vy;  0x8XY0 => vx := vy, //copy?, mov
            (8, x, y, 0) => {
                self.regs[x as usize] = self.regs[y as usize];
            }
            // OR Vx, Vy;  0x8XY1 => vx |= vy, //bit OR
            (8, x, y, 1) => {
                self.regs[x as usize] |= self.regs[y as usize];
            }
            // AND Vx, Vy;  0x8XY2 => vx &= vy, //bit AND
            (8, x, y, 2) => {
                self.regs[x as usize] &= self.regs[y as usize];
            }
            // XOR Vx, Vy;  0x8XY3 => vx ^= vy, //bit XOR
            (8, x, y, 3) => {
                self.regs[x as usize] ^= self.regs[y as usize];
            }
            // ADD Vx, Vy;  0x8XY4 => vx += vy, vf := CARRY BIT
            (8, x, y, 4) => {
                let result: u16 = self.regs[x as usize] as u16 + self.regs[y as usize] as u16;
                self.regs[x as usize] = (result & 0x00ff) as u8;
                if result > 0xff {
                    self.regs[0xf] = 1;
                } else {
                    self.regs[0xf] = 0;
                }
            }
            // SUB Vx, Vy;  0x8XY5 => vx -= vy, vf := NOT BORROW BIT
            (8, x, y, 5) => {
                if self.regs[x as usize] >= self.regs[y as usize] {
                    self.regs[0xf] = 1;
                } else {
                    self.regs[0xf] = 0;
                }

                self.regs[x as usize] = self.regs[x as usize].wrapping_sub(self.regs[y as usize]);
                // FIXME: address substraction with borrow. (wrapping_sub ??)
            }
            // SHR Vx, Vy;  0x8XY6 => Vx = Vy >> 1, Vf := old least significant bit of Vy
            (8, x, y, 6) => {
                self.regs[0xf] = self.regs[y as usize] & 1;
                self.regs[x as usize] = self.regs[y as usize] >> 1;
            }
            // SUBN Vx, Vy;  0x8XY7 => vx =- vy, vf := NOT BORROW BIT
            (8, x, y, 7) => {
                if self.regs[y as usize] > self.regs[x as usize] {
                    self.regs[0xfusize] = 1;
                } else {
                    self.regs[0xfusize] = 0;
                }

                self.regs[x as usize] = self.regs[y as usize] - self.regs[x as usize];
                // FIXME: address substraction with borrow. (wrapping_sub??)
            }
            // SHL Vx, Vy;  0x8XYE => Vx = Vy << 1, Vf := old most significant bit of Vy
            (8, x, y, 0xe) => {
                if self.regs[y as usize] & 0x80 > 0 {
                    self.regs[0xf] = 1;
                } else {
                    self.regs[0xf] = 0;
                }
                self.regs[x as usize] = self.regs[y as usize] << 1;
            }
            // SNE Vx, Vy;  0x9XY0 => if vx != vy then skip next instruction,
            (9, x, y, 0) => {
                if self.regs[x as usize] != self.regs[y as usize] {
                    self.pc += 2;
                }
            }
            // LD I, addr;  0xANNN => i := NNN, // assign to index 16bit register
            (0xa, _a, _b, _c) => {
                // self.index = addr(a, b, c);
                self.index = op & 0x0fff;
            }
            // JP V0, addr;  0xBNNN => jump0 NNN, // jump to address NNN + v0
            (0xb, _a, _b, _c) => {
                // self.pc = self.regs[0] as u16 + addr(a, b, c);
                self.pc = self.regs[0] as u16 + op & 0x0fff;
            }
            // RND Vx, byte;  0xCXNN => vx := random NN, // random number 0-255 AND NN
            (0xc, x, _a, _b) => {
                self.regs[x as usize] = rand::thread_rng().gen::<u8>() & op as u8;
                println!("RND & {:02x}: {}", op as u8, self.regs[x as usize]);
                // byte(a, b);
                // TODO: check that rand::thread_rng() is cheap
            }
            // DRW Vx, Vy, nibble;  0xDXYN => sprite vx vy N, //vf = 1 on collision
            (0xd, x, y, n) => {
                self.drw(x, y, n);
            }
            // SKP Vx;  0xEX9E => if vx -key then, //is a key not pressed?
            (0xe, x, 9, 0xe) => {
                if Chip8::key_pressed(self.regs[x as usize]) {
                    self.pc += 2;
                }
            }
            // SKNP Vx;  0xEXA1 => if vx key then, //is a key pressed?
            (0xe, x, 0xa, 1) => {
                if !Chip8::key_pressed(self.regs[x as usize]) {
                    self.pc += 2;
                }
            }
            // LD Vx, DT;  0xFX07 => vx := delay
            (0xf, x, 0, 7) => {
                self.regs[x as usize] = self.delay_timer;
            }
            // LD Vx, K;  0xFX0A => vx := key, //wait for a keypress
            (0xf, x, 0, 0xa) => {
                self.regs[x as usize] = Chip8::wait_key_press();
            }
            // LD DT, Vx;  0xFX15 => delay := vx,
            (0xf, x, 1, 5) => {
                self.delay_timer = self.regs[x as usize];
            }
            // LD ST, Vx;  0xFX18 => buzzer := vx,
            (0xf, x, 1, 8) => {
                self.sound_timer = self.regs[x as usize];
            }
            // ADD I, Vx;  0xFX1E => i += vx, //index += vx
            (0xf, x, 1, 0xe) => {
                self.index += self.regs[x as usize] as u16;
            }
            // LD F, Vx;  0xFX29 => i := hex vx, //set i to a hex character
            (0xf, x, 2, 9) => {
                let mut char_index = self.regs[x as usize];
                if char_index > 15 {
                    // TODO
                    eprintln!(
                        "TODO. THERE IS NOT BITMAP FONT FOR char_index > 15. char_index: {}",
                        char_index
                    );
                    // continue;
                    char_index %= 16;
                }
                self.index = char_index as u16 * 5; // the font is 4 bytes for 1 char
                println!("Fx29: new chip.index: {}", self.index);
                // unimplemented!();
            }
            // LD B, Vx;  0xFX33 => bcd vx, //decode vx into binary-coded decimal
            (0xf, x, 3, 3) => {
                let hundreds = self.regs[x as usize] / 100;
                let tens = (self.regs[x as usize] % 100) / 10;
                let ones = self.regs[x as usize] % 10;

                self.memory[self.index as usize] = hundreds;
                self.memory[(self.index + 1) as usize] = tens;
                self.memory[(self.index + 2) as usize] = ones;
            }
            // LD [I], Vx;  0xFX55 => save vx, //save v0-vx to i through (i+x)
            (0xf, x, 5, 5) => {
                for i in 0..=x {
                    self.memory[(self.index + i as u16) as usize] = self.regs[i as usize];
                }
                self.index += (x + 1) as u16;
            }
            // LD Vx, [I];  0xFX65 => load vx, //load v0-vx from i through (i+x)
            (0xf, x, 6, 5) => {
                for i in 0..=x {
                    self.regs[i as usize] = self.memory[(self.index + i as u16) as usize];
                }
                self.index += (x + 1) as u16;
            }
            _ => panic!("unimplemented: {:04x}", op),
        }
    }

    pub fn process_instructions(&mut self) {
        let mut w = crate::window::create_window();

        let mut counter = 0;
        loop {
            self.process_instruction();
            std::thread::sleep(std::time::Duration::from_millis(50));

            if counter == 100 {
                counter = 0;
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
            }
            counter += 1;

            let db = self.get_display_buffer();
            w.update_with_buffer(&db, DISPLAY_WIDTH, DISPLAY_HEIGHT)
                .expect("failed to update window");
        }
    }

    fn key_pressed(key: u8) -> bool {
        if key > 15 {
            panic!("key_pressed called with impossible key: 0x{:02x}", key);
        }
        KEYBOARD[(key & 0x0f) as usize]
    }

    fn wait_key_press() -> u8 {
        loop {
            print!("wait_key_press. which key you would like to press (0-16): ");
            std::io::stdout().flush().expect("failed to flush stdout");
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .expect("failed to read digits");
            let input = input.trim();
            let res = input.parse::<u8>();
            if res.is_err() {
                println!("failed to parse your input");
                println!("input should be 0..15");
                continue;
            }
            let res = res.unwrap();
            if res > 15 {
                println!("input should be 0..15");
                continue;
            }

            return res;
        }
    }

    // TODO: FIXME: get rid of Vec allocation
    fn get_display_buffer(&self) -> Vec<u32> {
        let mut v = Vec::with_capacity(DISPLAY_WIDTH * DISPLAY_HEIGHT);
        for x in 0..DISPLAY_WIDTH_IN_BYTES * DISPLAY_HEIGHT {
            let b = self.memory[VIDEO_MEMORY_OFFSET + x];
            let mut mask = 0x80u8;
            while mask > 0 {
                if b & mask > 0 {
                    v.push(0x00AAAA00u32);
                } else {
                    v.push(0x00000000u32);
                }
                mask >>= 1;
            }
        }
        v
    }

    // DRW Vx, Vy, nibble;  0xDXYN => sprite vx vy N, //vf = 1 on collision
    // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and
    // a height of N pixels. Each row of 8 pixels is read as bit-coded starting
    // from memory location I; I value does not change after the execution of
    // this instruction. As described above, VF is set to 1 if any screen pixels
    // are flipped from set to unset when the sprite is drawn, and to 0 if that
    // does not happen.
    fn drw(&mut self, x: u8, y: u8, n: u8) {
        // println!("Draw at (x: {x}, y: {y}), self.index: 0x{:02x}/{}", self.index, self.index);
        // println!("self.index: 0x{:02x}/{}", self.index, self.index);

        let mut vf = 0; // 1 - if any pixel flipped from set to unset when the sprite is drawn
                        // 0 - otherwise
        let x = self.regs[x as usize] % 64; // TODO: add optional wrap-around X
        let y = self.regs[y as usize] % 32; // TODO: add optional wrap-around y

        /* for i in 0..n as usize {
            let sprite_byte = self.memory[self.index as usize + i];
            let byte_offset = x / 8;
            let bit_offset = x % 8;
            let mut mask = 0x80;
            while mask > 0 {
                if sprite_byte & mask > 0 {
                    xor_bit();
                }
                mask >>= 1;
            }
        } */

        let rem = x % 8;
        for i in 0..n as usize {
            let mut byte_offset =
                VIDEO_MEMORY_OFFSET + y as usize * DISPLAY_WIDTH_IN_BYTES + x as usize;
            // println!("byte_offset: {}, x: {}, y: {}, i: {}", byte_offset, x, y, i);
            let display_byte = self.memory[byte_offset];
            let sprite_byte = self.memory[self.index as usize + i] >> rem;
            self.memory[byte_offset] = display_byte ^ sprite_byte;

            if display_byte & sprite_byte > 0 {
                vf = 1;
            }

            if rem > 0 {
                byte_offset += 1;
                let display_byte = self.memory[byte_offset];
                let sprite_byte = self.memory[self.index as usize + i] << rem;
                self.memory[byte_offset] = display_byte ^ sprite_byte;

                if display_byte & sprite_byte > 0 {
                    vf = 1;
                }
            }
        }

        self.regs[0xf] = vf;
        // std::thread::sleep(std::time::Duration::from_millis(50));
        // self.draw_frame_buffer();
    }

    fn draw_frame_buffer(&self) {
        // const VIDEO_MEM_OFFSET: usize = 0xF00;
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH_IN_BYTES {
                let mut mask: u8 = 0x80;
                let byte = self.memory[VIDEO_MEMORY_OFFSET + row * DISPLAY_WIDTH_IN_BYTES + col];
                while mask > 0 {
                    if byte & mask > 0 {
                        print!("##");
                    } else {
                        print!("  ");
                    }
                    mask >>= 1;
                }
            }
            println!();
        }
    }
}

fn nnn(a: u8, b: u8, c: u8) -> u16 {
    (a as u16) << 8 | (b as u16) << 4 | (c as u16)
}

fn addr(a: u8, b: u8, c: u8) -> u16 {
    nnn(a, b, c)
}

fn kk(a: u8, b: u8) -> u8 {
    (a << 4) | b
}

fn byte(a: u8, b: u8) -> u8 {
    kk(a, b)
}

// TODO: FIXME: Add 60Hz timer.

#[cfg(test)]
mod tests {
    use crate::vm::*;

    #[test]
    #[allow(non_snake_case)]
    fn test_0x7XNN_add() {
        let mut chip8 = Chip8::new();

        let instructions: &[u8] = &[
            0x60, 0x20, // ld  v0, 0x20
            0x70, 0x40, // add v0, 0x40
            0x62, 0x90, // ld  v2, 0x90
            0x80, 0x24, // add v0, v2
        ];

        // populate instructions
        chip8.load_rom(instructions);

        let mut expected = Chip8::new();
        expected.regs[0] = 0xF0;
        expected.regs[2] = 0x90;

        chip8.process_instruction();
        assert_eq!(chip8.regs[0], 0x20);

        chip8.process_instruction();
        assert_eq!(chip8.regs[0], 0x60);

        chip8.process_instruction();
        assert_eq!(chip8.regs[0], 0x60);
        assert_eq!(chip8.regs[2], 0x90);

        chip8.process_instruction();
        assert_eq!(chip8.regs[0], 0xF0);
        assert_eq!(chip8.regs[2], 0x90);

        assert_eq!(chip8.regs, expected.regs);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_0x8XY6_shr() {
        let mut chip8 = Chip8::new();
        let instructions: &[u8] = &[
            0x63, 0x92, // ld v3, 0x92 ; this one is just to test v3 afterwards
            0x64, 0x55, // ld v4, 0x55
            0x83, 0x46, // shr v3, v4
        ];

        chip8.load_rom(&instructions);

        chip8.process_instruction(); // ld v3, 0x92
        assert_eq!(chip8.regs[3], 0x92);
        assert_eq!(chip8.regs[0xf], 0);

        chip8.process_instruction(); // ld v4, 0x55
        assert_eq!(chip8.regs[4], 0x55);
        assert_eq!(chip8.regs[0xf], 0);

        chip8.process_instruction(); // shr v3, v4
        assert_eq!(chip8.regs[4], 0x55);
        assert_eq!(chip8.regs[3], 0x55u8 >> 1);
        assert_eq!(chip8.regs[0xf], 1);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_0x8XYe_shl() {
        let mut chip8 = Chip8::new();
        let instructions: &[u8] = &[
            0x63, 0x92, // ld  v3, 0x92 ; this one is just to test v3 afterwards
            0x64, 0x80, // ld  v4, 0x80
            0x83, 0x4e, // shl v3, v4
            0x64, 0x7a, // ld  v4, 0x7a
            0x83, 0x4e, // shl v3, v4
        ];

        chip8.load_rom(&instructions);

        chip8.process_instruction(); // ld v3, 0x92
        assert_eq!(chip8.regs[3], 0x92);
        assert_eq!(chip8.regs[0xf], 0);

        chip8.process_instruction(); // ld v4, 0x80
        assert_eq!(chip8.regs[4], 0x80);
        assert_eq!(chip8.regs[0xf], 0);

        chip8.process_instruction(); // shl v3, v4
        assert_eq!(chip8.regs[4], 0x80);
        assert_eq!(chip8.regs[3], 0);
        assert_eq!(chip8.regs[0xf], 1);

        chip8.process_instruction(); // ld v4, 0x7a
        assert_eq!(chip8.regs[4], 0x7a);
        assert_eq!(chip8.regs[3], 0);
        assert_eq!(chip8.regs[0xf], 1);

        chip8.process_instruction(); // shl v3, v4
        assert_eq!(chip8.regs[4], 0x7a);
        assert_eq!(chip8.regs[3], 0x7au8 << 1);
        assert_eq!(chip8.regs[0xf], 0);
    }
}
