use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

/// General chip 8 struct
pub struct Chip8 {
    register: [u8; 16],
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
    /// * `register`: all zeroes
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
            register: [0; 16],
            memory: [0; 4096],
            index_register: 0,
            program_counter: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [[0; 64]; 32],
            op_code: 0,
        }
    }
    
    pub fn load_rom(&mut self, path: PathBuf) {
        // memory addres before this are reserved
        let start_address = 0x200; 

        // open rom file
        let file = File::open("roms/Chip8-Picture.ch8").expect("Error opening rom file");

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

    fn load_fonts(mut self) {
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
}