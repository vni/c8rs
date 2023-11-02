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
    sp: u16,
    sound_timer: u8, // buzzes while activated. Decreases at 60Hz. When 0 -> deactivates.
    delay_timer: u8, // decreases till 0 at rate of 60Hz. When it 0 -> stops.
    memory: [u8; 4096],

    halt: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            regs: [0u8; 16],
            index: 0,
            pc: 0x200,
            sp: 0xEA0, // 0xFA0 ??
            sound_timer: 0,
            delay_timer: 0,
            memory: [0u8; 4096],
            halt: false,
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

        const VF: usize = 0xf;

        match op >> 12 {
            0x0 => {
                match op {
                    0x00e0 => {
                        // CLS: clear the display
                        for i in 0xF00..=0xFFF {
                            self.memory[i] = 0u8;
                        }
                    }

                    0x00ee => {
                        // RET: return
                        debug_assert_eq!(self.sp & 1, 0);
                        self.sp -= 2; // FIXME: check for underflow
                        self.pc = (self.memory[self.sp as usize] as u16) << 8
                            | self.memory[self.sp as usize + 1] as u16;
                    }

                    _ => {
                        // SYS addr;  obsolete
                        unimplemented!("`SYS addr` instruction is obsolete");
                    }
                }
            }

            0x1 => {
                // 0x1NNN jump NNN
                // check for halt instruction
                if (self.pc - 2) == (op & 0x0fff) {
                    self.halt = true;
                }
                self.pc = op & 0x0fff;
            }

            0x2 => {
                // 0x2NNN call NNN
                debug_assert_eq!(self.sp & 1, 0);
                self.memory[self.sp as usize] = (self.pc >> 8) as u8;
                self.memory[self.sp as usize + 1] = self.pc as u8;
                self.sp += 2;
                self.pc = op & 0x0fff;
            }

            0x3 => {
                // 0x3XNN SE Vx, byte; if vx == kk => skip next instruction
                let x: usize = (op as usize >> 8) & 0xf;
                let nn: u8 = (op & 0xff) as u8;
                if self.regs[x] == nn {
                    self.pc += 2;
                }
            }

            0x4 => {
                // 0x4XNN SNE Vx, byte; if vx != kk => skip next instruction
                let x: usize = (op as usize >> 8) & 0xf;
                let nn: u8 = (op & 0xff) as u8;
                if self.regs[x] != nn {
                    self.pc += 2;
                }
            }

            0x5 => {
                // 0x5XY0 SE Vx, Vy;  0x5XY0 => if vx == vy => skip next instruction
                let x: usize = (op as usize >> 8) & 0xf;
                let y: usize = (op as usize >> 4) & 0xf;
                if self.regs[x] == self.regs[y] {
                    self.pc += 2;
                }
            }

            0x6 => {
                // 0x6XNN ld vx, nn
                let x: usize = (op as usize >> 8) & 0xf;
                let nn: u8 = (op & 0xff) as u8;
                self.regs[x] = nn;
            }

            0x7 => {
                // 0x7XNN add vx, nn
                let x: usize = (op as usize >> 8) & 0xf;
                let nn: u8 = (op & 0xff) as u8;
                self.regs[x] = self.regs[x].wrapping_add(nn);
            }

            0x8 => {
                let x: usize = (op as usize >> 8) & 0xf;
                let y: usize = (op as usize >> 4) & 0xf;

                match op & 0xf {
                    0x0 => {
                        // 0x8XY0 ld Vx, Vy // vx := vy //copy?, mov
                        self.regs[x] = self.regs[y];
                    }

                    0x1 => {
                        // 0x8XY1 or Vx, Vy; // vx |= vy //bit OR
                        self.regs[x] |= self.regs[y];
                    }

                    0x2 => {
                        // 0x8XY2 and Vx, Vy // vx &= vy //bit AND
                        self.regs[x as usize] &= self.regs[y as usize];
                    }

                    0x3 => {
                        // 0x8XY3 xor Vx, Vy // vx ^= vy //bit XOR
                        self.regs[x] ^= self.regs[y];
                    }

                    0x4 => {
                        // 0x8XY4 add Vx, Vy // vx += vy, vf := carry_bit
                        let result: u16 = self.regs[x] as u16 + self.regs[y] as u16;
                        self.regs[x] = result as u8;
                        if result > 0xff {
                            self.regs[VF] = 1;
                        } else {
                            self.regs[VF] = 0;
                        }
                    }

                    0x5 => {
                        // 0x8XY5 sub Vx, Vy // vx -= vy, vf := not_borrow_bit
                        if self.regs[x] >= self.regs[y] {
                            self.regs[VF] = 1;
                        } else {
                            self.regs[VF] = 0;
                        }

                        self.regs[x] = self.regs[x].wrapping_sub(self.regs[y]);
                        // FIXME: TODO: check the logic
                    }

                    0x6 => {
                        // 0x8XY6 shr Vx, Vy // Vx = Vy >> 1, Vf := old least significant bit of Vy
                        self.regs[VF] = self.regs[y] & 1;
                        self.regs[x] = self.regs[y] >> 1;
                    }

                    0x7 => {
                        // 0x8XY7 subn Vx, Vy // vx =- vy, vf := not_borrow_bit
                        if self.regs[y] > self.regs[x] {
                            self.regs[VF] = 1;
                        } else {
                            self.regs[VF] = 0;
                        }

                        self.regs[x as usize] = self.regs[y as usize] - self.regs[x as usize];
                        // FIXME: address substraction with borrow. (wrapping_sub??)
                    }

                    0xE => {
                        // 0x8XYE shl Vx, Vy // Vx = Vy << 1, Vf := old most significant bit of Vy
                        if self.regs[y] & 0x80 > 0 {
                            self.regs[VF] = 1;
                        } else {
                            self.regs[VF] = 0;
                        }
                        self.regs[x] = self.regs[y] << 1;
                    }

                    _ => unimplemented!("{:04x} sub instruction of 0x8XY.. is not implemented", op),
                }
            }

            0x9 => {
                // 0x9XY0 SNE Vx, Vy;  0x9XY0 => if vx != vy then skip next instruction
                let x: usize = (op as usize >> 8) & 0xf;
                let y: usize = (op as usize >> 4) & 0xf;
                if self.regs[x] != self.regs[y] {
                    self.pc += 2;
                }
            }

            0xA => {
                // 0xANNN ld I, nnn // assign to index 16bit register
                self.index = op & 0x0fff;
            }

            0xB => {
                // 0xBNNN jmp0 nnn // jump to address v0 + NNN
                self.pc = self.regs[0] as u16 + op & 0x0fff;
            }

            0xC => {
                // 0xCXNN rnd Vx, nn // random(0, 255) && nn
                let x: usize = (op as usize >> 8) & 0xf;
                self.regs[x] = rand::thread_rng().gen::<u8>() & (op as u8);
                println!("RND & {:02x}: {}", op as u8, self.regs[x]);
                // TODO: check that rand::thread_rng() is cheap
            }

            0xD => {
                // 0xDXYN drw vx, vy, nibble // sprite vx vy N, // vf = 1 on collision
                let x: u8 = ((op >> 8) & 0xf) as u8;
                let y: u8 = ((op >> 4) & 0xf) as u8;
                let n: u8 = (op & 0xf) as u8;

                self.drw(x, y, n);
            }

            0xE => {
                let x: usize = (op as usize >> 8) & 0xf;

                match op & 0xff {
                    0x9E => {
                        // 0xEX9E skp Vx, K // if vx -key then // is key pressed?
                        if Chip8::key_pressed(self.regs[x]) {
                            self.pc += 2;
                        }
                    }

                    0xA1 => {
                        // 0xEXA1 sknp Vx, K // if vx key then // is key not pressed?
                        if !Chip8::key_pressed(self.regs[x]) {
                            self.pc += 2;
                        }
                    }

                    _ => unimplemented!("{:04x} subinstruction of 0xEX.. is not implemented", op),
                }
            }

            0xF => {
                let x: usize = (op as usize >> 8) & 0xf;

                match op & 0xff {
                    0x07 => {
                        // 0xFX07 ld Vx, dt // vx := delay
                        self.regs[x] = self.delay_timer;
                    }

                    0x0A => {
                        // 0xFX0A ld Vx, K // vx := key //wait for a keypress
                        self.regs[x] = Chip8::wait_key_press();
                    }

                    0x15 => {
                        // 0xFX15 ld dt, Vx // delay := vx
                        self.delay_timer = self.regs[x];
                    }

                    0x18 => {
                        // 0xFX18 ld st, Vx // buzzer := vx
                        self.sound_timer = self.regs[x];
                    }

                    0x1E => {
                        // 0xFX1E add I, Vx // i += vx //index += vx
                        self.index += self.regs[x] as u16;
                    }

                    0x29 => {
                        // 0xFX29 ld F, Vx // i := hex vx //set i to a hex character
                        let mut char_index = self.regs[x];
                        if char_index > 15 {
                            // TODO
                            eprintln!(
                                "TODO. THERE IS NOT BITMAP FONT FOR char_index > 15. char_index: {}",
                                char_index
                            );
                            // continue;
                            char_index %= 16;
                        }
                        self.index = char_index as u16 * 5; // the font is 5 bytes toll for 1 char
                        println!("Fx29: new chip.index: {}", self.index);
                    }

                    0x33 => {
                        // 0xFX33 bcd vx // decode vx into binary-coded decimal
                        let hundreds = self.regs[x] / 100;
                        let tens = (self.regs[x] % 100) / 10;
                        let ones = self.regs[x] % 10;

                        self.memory[self.index as usize] = hundreds;
                        self.memory[(self.index + 1) as usize] = tens;
                        self.memory[(self.index + 2) as usize] = ones;
                    }

                    0x55 => {
                        // 0xFX55 LD [I], Vx // save vx //save v0-vx to i through (i+x)
                        for i in 0..=x {
                            self.memory[(self.index + i as u16) as usize] = self.regs[i as usize];
                        }
                        self.index += (x + 1) as u16;
                    }

                    0x65 => {
                        // 0xFX65 ld Vx, [I] // load vx //load v0-vx from i through (i+x)
                        for i in 0..=x {
                            self.regs[i as usize] = self.memory[(self.index + i as u16) as usize];
                        }
                        self.index += (x + 1) as u16;
                    }

                    _ => unimplemented!("{:04x} sub instruction of 0xFX.. is not implemented", op),
                }
            }

            _ => panic!("unimplemented: {:04x}", op),
        }
    }

    pub fn process_instructions(&mut self) {
        let mut w = crate::window::create_window();

        let mut counter = 0;
        while self.halt == false {
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
