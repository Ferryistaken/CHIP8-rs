#![allow(non_snake_case)]

use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use rand::Rng;
use std::convert::TryFrom;

/// General chip 8 struct
pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index_register: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [[u32; 64]; 32],
    op_code: u16,
}

impl Chip8 {
    /// Create a new chip8 instance with an empty `rom`, ready for use
    ///
    /// ## Values
    ///
    /// * `registers`: all zeroes
    /// * `memory`/`rom`: all zeroes
    /// * `index_register`: 0
    /// * `program_counter`: 0x200
    /// * `stack`: all zeroes
    /// * `stack_poionter`: 0
    /// * `delay_timer`: 0
    /// * `sound_timer`: 0
    /// * `keypad`: all zeroes
    /// * `video`: all zeroes
    /// * `op_code`: 0
    /// * `fontset_size`: 80
    pub fn new() -> Self {
        Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index_register: 0,
            program_counter: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [[15; 64]; 32],
            op_code: 0,
        }
    }

    pub fn load_rom(&mut self, path: PathBuf) {
        // memory addres before this are reserved
        let start_address = 0x200; 

        // open rom file
        let file = File::open(path).expect("Error opening rom file");

        // buffer to store bytes in
        let mut buf: Vec<u8> = Vec::new();

        // read the bytes from file
        for byte in file.bytes() {
            let byte = match byte {
                Ok(byte) => byte,
                Err(error) => panic!("Provided rom is not a valid binary. {:?}", error),
            };
            buf.push(byte);
        }
        // println!("{:x?}", buf);

        // load the buffer into the chip 8 memory
        for i in 0..buf.len() - 1 {
            self.memory[start_address + i] = buf[i];
        }
    }

    /// Returns `byte_number` amount of bytes after the `pointer` in rom
    pub fn dump_rom(&self, start_address: usize, byte_number: usize) -> Vec<u8> {
        let mut rom: Vec<u8> = Vec::new();
        // for each memory address after the pointer add that address to the buffer, then return the buffer
        for i in start_address..start_address + byte_number {
            rom.push(self.memory[i]);
        }
        return rom;
    }

    pub fn dump_video(&mut self) -> Vec<&u32> {
        let mut buf: Vec<&u32> = Vec::new();
        for (_i, row) in self.video.iter_mut().enumerate() {
            for (_y, col) in row.iter_mut().enumerate() {
                println!("{}", col);
                buf.push(col);
            }
        }
        return buf;
    }

    /// Load the fontset into memory
    fn load_fonts(mut self) {
        // TODO: load fonts in the constructor
        let fontset: [u8; 80] = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F];
        ];

        let fontset_start_address = 0x50;

        for i in 0..fontset.len() - 1 {
            self.memory[fontset_start_address + i as usize] = fontset[i as usize];
        }
    }

    /// Generate a random byte(0, 255)
    pub fn rand_byte(&self) -> u8 {
        let mut rng = rand::thread_rng();
        return rng.gen_range(0, 255);
    }

    //
    //
    // From Now On I will define all `opcodes`, taken from here:
    //
    // https://austinmorlan.com/posts/chip8_emulator/
    //
    //

    /// OPCODE 00E0 - Clear Screen
    pub fn OP_00E0(&mut self) {
        // set video buffer to zero
        // memset doesn't work
        // self.video.iter_mut().for_each(|m| *m = [0; 64])
        self.video = [[0; 64]; 32];
    }

    /// OPCODE 00EE - Return from subroutine
    fn OP_00EE(&mut self) {
        self.stack_pointer = self.stack_pointer - 1;

        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    /// OPCODE 1NNN - Jump to location NNN(set program counter to nnn)
    fn OP_1nnn(&mut self) {
        // using 0x0FFF I can take the NNN from the opcode while leaving the one
        let address: u16 = self.op_code & 0x0FFF;

        self.program_counter = address;
    }

    /// OPCODE 2NNN - Call subroutine at location NNN
    fn OP_2nnn(&mut self) {
        let address: u16 = self.op_code & 0x0FFF;

        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer = self.stack_pointer + 1;
        self.program_counter = address;
    }

    // TODO: Make all of these `Vx` Variables `u8` instead of `u16`

    /// OPCODE 3XKK - Skip next instruction if Vx = kk
    /// Since our PC has already been incremented by 2 in Cycle(), we can just increment by 2 again to skip the next instruction.
    fn OP_3xkk(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00) >> 8;
        let byte: u16 = self.op_code & 0x00FF;

        let byte: u8 = match u8::try_from(byte) {
            Ok(number) => number,
            Err(error) => panic!("Could not turn u16 into u8 in OPCODE: 3XKK. Error: {}", error),
        };

        if self.registers[Vx as usize] == byte {
            self.program_counter += 2;
        }
    }

    /// OPCODE 4XKK - Skip next instruction if Vx != kk
    fn OP_4xkk(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00) >> 8;
        let byte: u16 = self.op_code & 0x00FF;

        let byte: u8 = match u8::try_from(byte) {
            Ok(number) => number,
            Err(error) => panic!("Could not turn u16 into u8 in OPCODE: 3XKK. Error: {}", error),
        };

        // this != is the onlh difference from the function above
        if self.registers[Vx as usize] != byte {
            self.program_counter += 2;
        }
    }

    /// OPCODE 5XY0 - Skip next instruction if Vx = Vy.
    fn OP_5xy0(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00) >> 8;
        let Vy: u16 = (self.op_code & 0x00F0) >> 4;

        if self.registers[Vx as usize] == self.registers[Vy as usize] {
            self.program_counter += 2;
        }
    }
}
