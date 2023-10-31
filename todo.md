- [ ] add support for status register vf
- [ ] find and add roms for chip8 to test vm and disassembler
- [ ] make the disassembler understand every instruction
- [ ] make the vm understand every instruction
      - [ ] fix todo, fixme, unimplemented!() in code
      - [ ] check that every instruction works correctly
- [ ] add unit tests for the vm instructions (at least, for the majority of them)
- [ ] add support for other chip8 alternatives (super-chip, chip48, ...)

- [ ] all the available memory to the vm should be in 4096 bytes (including lower 512 bytes for 'bios', display memory and stack)
      - [ ] make the display to be in memory[0xF00 - 0xFFF] (256 bytes) instead being a separate memory
      - [ ] make the stack to be in memory[0xEA0 - 0xEFF] (96 bytes) instead of being a separate memory

- [ ] use .rustfmt config instead of #[rustfmt::skip] in code