// This may be a standalone project
// standalone binary + standalone library

// TODO
// add labels
// add comments
// add directives
// parse simple arguments

/*
https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set


0x0NNN - Execute machine language subroutine at address NNN
    unused nowadays
0x00E0 - CLS cls              -- Clear the screen
0x00EE - RET ret              -- Return from a subroutine
0x1NNN - JMP jmp nnn          -- Jump to address NNN
0x2NNN - CALL call nnn        -- Execute subroutine starting at address NNN
0x3XNN - SKPE? skpe? vx, nn   -- Skip the following instruction if the value of register VX equals NN
0x4XNN - SKNE? skne? vx, nn   -- Skip the following instruction if the value of register VX is not equal to NN
0x5XY0 - SKP vx, vy           -- Skip the following instruction if the value of register VX is equal to the value of register VY
0x6XNN - LD ld vx, nn         -- Store number NN in register VX
0x7XNN - ADD add vx, nn       -- Add the value NN to register VX

0x8XY0 - ld vx, vy            -- Store the value of register VY in register VX
0x8XY1 - or vx, vy            -- Set VX to VX OR VY
0x8XY2 - and vx, vy           -- Set VX to VX AND VY
0x8XY3 - xor vx, vy           -- Set VX to VX XOR VY
0x8XY4 - adc vx, vy           -- Add the value of register VY to register VX
                                 Set VF to 01 if a carry occurs
                                 Set VF to 00 if a carry does not occur
0x8XY5 - sub vx, vy           -- Subtract the value of register VY from register VX
                                 Set VF to 00 if a borrow occurs
                                 Set VF to 01 if a borrow does not occur
0x8XY6 - shr vx, vy           -- Store the value of register VY shifted right one bit in register VX¹
                                 Set register VF to the least significant bit prior to the shift VY is unchanged
0x8XY7 - subn vx, vy, vx      -- Set register VX to the value of VY minus VX
                                 Set VF to 00 if a borrow occurs
                                 Set VF to 01 if a borrow does not occur
0x8XYE - shl vx, vy           -- Store the value of register VY shifted left one bit in register VX¹
                                 Set register VF to the most significant bit prior to the shift VY is unchanged

0x9XY0 - SKPNE skpne vx, vy   -- Skip the following instruction if the value of register VX is not equal to the value of register VY
0xANNN - LD ld I, nnn         -- Skip the following instruction if the value of register VX is not equal to the value of register VY
0xBNNN - JMP jmp v0, nnn      -- Jump to address NNN + V0
0xCXNN - RND rnd vx, nn       -- Set VX to a random number with a mask of NN
0xDXYN - DRW drw vx, vy, n    -- Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I
                              -- Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
0xEX9E - SKP vx, K            -- Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed
0xEXA1 - SKR vx, K            -- Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed

0xFX07 - LD ld vx, dt         -- Store the current value of the delay timer in register VX
0xFX0A - wait vx, K           -- Wait for a keypress and store the result in register VX
0xFX15 - LD ld dt, vx         -- Set the delay timer to the value of register VX
0xFX18 - LD ld st, vx         -- Set the sound timer to the value of register VX
0xFX1E - ADD I, vx            -- Add the value stored in register VX to register I
0xFX29 - LD I, ???            -- Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX
0xFX33 - BCD vx               -- Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2
0xFX55 - store vx             -- Store the values of registers V0 to VX inclusive in memory starting at address I
I is set to I + X + 1 after operation²
0xFX65 - load vx              -- Fill registers V0 to VX inclusive with the values stored in memory starting at address I
I is set to I + X + 1 after operation²
*/

#[rustfmt::skip]
const MNEMONICS: &[&str] = &["cls", "ret", "sys", "jp",
                             "call", "se", "sne", "ld",
                             "add", "or", "and", "xor",
                             "sub", "shr", "subn", "shl",
                             "rnd", "drw", "skp", "sknp"];

// TODO: add comments support
// TODO: add labels support
// TODO: add arguments support
// FIXME: remove this next line
#[allow(dead_code)]
pub fn asm(mut s: &str) -> Result<u16, &str> {
    // 0) read from s line by line
    // + 1) remove comments
    // + 2) trim
    // 3) skip if empty
    // 4) try to get `label:`
    // 5) get mnemonic

    if let Some(x) = s.find(';') {
        s = &s[..x];
    }

    let lowercased = s.trim().to_lowercase();
    if lowercased.is_empty() {
        return Err("Empty string");
    }

    let mut splitted = lowercased.split_whitespace();
    let mnemonic = splitted.next().unwrap();
    let mut label = None;

    if !MNEMONICS.contains(&mnemonic) {
        label = Some(mnemonic);
    }

    if let Some(lbl) = label {
        println!("label: {}", lbl);
    }

    match mnemonic {
        "cls" => {
            println!("cls");
            return Ok(0x00e0);
        }
        "ret" => {
            println!("ret");
            return Ok(0x00ee);
        }
        "sys" => {
            let addr = splitted.next().unwrap().parse::<u16>().unwrap();
            return Ok(0x0 | (addr & 0xfff));
        }
        "jp" | "jmp" => {
            println!("jp|jmp");
            let addr = splitted.next().unwrap().parse::<u16>().unwrap();
            return Ok(0x1000 | (addr & 0xfff));
        }
        "call" => {
            println!("call");
            let addr = splitted.next().unwrap().parse::<u16>().unwrap();
            return Ok(0x2000 | (addr & 0xfff));
        }
        "se" => {
            // se Vx, byte
            let x = splitted.next().unwrap().parse::<u8>().unwrap();
            assert!(x <= 0xf);
            let kk = splitted.next().unwrap().parse::<u8>().unwrap();
            return Ok(0x3000 | ((x as u16) << 8) | (kk as u16));

            // "se" => { // FIXME: se 0x3xkk
            //     return 0x5xy0 // se Vx, Vy
            // }
        }
        "sne" => {
            // sne Vx, byte
            let x = splitted.next().unwrap().parse::<u8>().unwrap();
            assert!(x <= 0xf);
            let kk = splitted.next().unwrap().parse::<u8>().unwrap();
            return Ok(0x4000 | ((x as u16) << 8) | (kk as u16));
        }
        "ld" => println!("ld"),
        "add" => println!("add"),
        "or" => {}
        "and" => {}
        "xor" => {}
        "sub" => {}
        "shr" => {}
        "subn" => {}
        "shl" => {}
        "rnd" => {}
        "drw" => {}
        "skp" => {}
        "sknp" => {}

        _ => println!("unknown!!"),
    }

    Ok(0x0000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_comment_removal() {
        let mut s = "  org 0x7c00 ; hello world. the comment goes here";
        if let Some(x) = s.find(';') {
            s = &s[..x];
        }
        assert_eq!(s, "  org 0x7c00 ");
        s = s.trim();
        assert_eq!(s, "org 0x7c00");
    }

    #[test]
    fn check_asm_0x0_cls_ret_sys() {
        assert_eq!(asm("cls"), 0x00e0);
        assert_eq!(asm("ret"), 0x00ee);
        // assert_eq!(asm("sys"))
    }

    #[test]
    fn check_asm_0x1_jp() {
        panic!();
        //
    }

    #[test]
    fn check_asm_0x2_call() {
        panic!();
        //
    }

    #[test]
    fn check_asm_0x3_se() {
        panic!();
        //
    }

    #[test]
    fn check_asm_0x4_sne() {
        panic!();
        //
    }
    #[test]
    fn check_asm_0x5_se() {
        panic!();
    }

    #[test]
    fn check_asm_0x6_ld() {
        panic!();
    }

    #[test]
    fn check_asm_0x7_add() {
        panic!();
    }

    #[test]
    fn check_asm_0x8_ld_or_and_xor_add_sub_shr_subn_shl() {
        panic!();
    }

    #[test]
    fn check_asm_0x9_sne() {
        panic!();
    }

    #[test]
    fn check_asm_0xa_ld() {
        panic!();
    }

    #[test]
    fn check_asm_0xb_jp() {
        panic!();
    }

    #[test]
    fn check_asm_0xc_rnd() {
        panic!();
    }
    #[test]
    fn check_asm_0xd_drw() {
        panic!();
    }

    #[test]
    fn check_asm_0xe_skp_sknp() {
        panic!();
    }

    #[test]
    fn check_asm_0xf_ld_add() {
        panic!();
    }
}
