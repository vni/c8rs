use std::io::Read;

pub fn chip8_disasm_main(filename: &str) {
    let mut buf: [u8; 4096] = [0; 4096];

    let mut f = std::fs::File::open(filename).expect("failed to open a file");
    loop {
        let n = f
            .read(&mut buf)
            .expect("failed to read bytes from input file");
        if n == 0 {
            break;
        }

        disasm(&buf);
    }
}

fn disasm(mut buf: &[u8]) {
    while buf.len() > 2 {
        let opcode = (buf[0] as u16) << 8 | buf[1] as u16;
        disasm_inst(opcode);
        buf = &buf[2..];
    }
}

fn create_nn(n1: u16, n2: u16) -> u16 {
    n1 << 4 | n2
}

fn create_nnn(n1: u16, n2: u16, n3: u16) -> u16 {
    n1 << 8 | n2 << 4 | n3
}

pub fn disasm_inst(opcode: u16) {
    let a = (opcode >> 12) & 0xf;
    let b = (opcode >> 8) & 0xf;
    let c = (opcode >> 4) & 0xf;
    let d = (opcode >> 0) & 0xf;
    match (a, b, c, d) {
        (0, 0, 0xE, 0) => {
            println!("CLS");
        }
        (0, 0, 0xE, 0xE) => {
            println!("RET");
        }
        (1, n1, n2, n3) => {
            let nnn = create_nnn(n1, n2, n3);
            println!("JMP 0x{:04x}/{}", nnn, nnn);
        }
        (2, n1, n2, n3) => {
            let nnn = create_nnn(n1, n2, n3);
            println!("CALL 0x{:04x}/{}", nnn, nnn);
        }
        (3, x, n1, n2) => {
            let nn = create_nn(n1, n2);
            println!("SE V{}, {}", x, nn);
        }
        (4, x, n1, n2) => {
            let nn = create_nn(n1, n2);
            println!("SNE V{}, {}", x, nn);
        }
        (5, x, y, 0) => {
            println!("SE V{}, V{}", x, y);
        }
        (6, x, n1, n2) => {
            let nn = create_nn(n1, n2);
            println!("LD V{}, {}", x, nn);
        }
        (7, x, n1, n2) => {
            let nn = create_nn(n1, n2);
            println!("ADD V{}, {}", x, nn);
        }
        (8, x, y, 0) => {
            println!("LD V{}, V{}", x, y);
        }
        (8, x, y, 1) => {
            println!("OR V{}, V{}", x, y);
        }
        (8, x, y, 2) => {
            println!("AND V{}, V{}", x, y);
        }
        (8, x, y, 3) => {
            println!("XOR V{}, V{}", x, y);
        }
        (8, x, y, 4) => {
            println!("ADD V{}, V{}", x, y);
        }
        (8, x, y, 5) => {
            println!("SUB V{}, V{}", x, y);
        }
        (8, x, _y, 6) => {
            // SHR VX {, VY} ???
            println!("SHR V{}", x);
        }
        (8, x, _y, 0xE) => {
            // SHR VX {, VY} ???
            println!("SHL V{}", x);
        }
        (9, x, y, 0) => {
            println!("SNE V{}, V{}", x, y);
        }
        (0xA, n1, n2, n3) => {
            let nnn = create_nnn(n1, n2, n3);
            println!("LD I, 0x{:04x}/{}", nnn, nnn);
        }
        (0xB, n1, n2, n3) => {
            let nnn = create_nnn(n1, n2, n3);
            println!("JMP V0, 0x{:04x}/{}", nnn, nnn);
        }
        (0xC, x, n1, n2) => {
            let nn = create_nn(n1, n2);
            println!("RND V{}, {}", x, nn);
        }
        (0xD, x, y, n) => {
            println!("DRW V{}, V{}, {}", x, y, n);
        }
        (0xE, x, 9, 0xE) => {
            println!("SKP V{}", x);
        }
        (0xE, x, 0xA, 1) => {
            println!("SKNP V{}", x);
        }
        (0xF, x, 0, 7) => {
            println!("LD V{}, DT", x);
        }
        (0xF, x, 0, 0xA) => {
            println!("LD V{}, K", x);
        }
        (0xF, x, 1, 5) => {
            println!("LD DT, V{}", x);
        }
        (0xF, x, 1, 8) => {
            println!("LD ST, V{}", x);
        }
        (0xF, x, 1, 0xE) => {
            println!("ADD I, V{}", x);
        }
        (0xF, x, 2, 9) => {
            println!("LD F, V{}", x);
        }
        (0xF, x, 3, 3) => {
            println!("LD B, V{}", x);
        }
        (0xF, x, 5, 5) => {
            println!("LD [I], V{}", x);
        }
        (0xF, x, 6, 5) => {
            println!("LD V{}, [I]", x);
        }
        _ => println!("UNKNOWN INSTRUCTION: {:x}{:x}{:x}{:x}", a, b, c, d),
    }
}
