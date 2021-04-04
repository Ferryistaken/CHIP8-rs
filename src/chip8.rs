#![allow(non_snake_case)]

use rand::Rng;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::ops::ShrAssign;
use function_name::named;

extern crate sdl2;

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
    video: [u32; 64 * 32],
    op_code: u16,
    table: [fn(&mut Chip8); 0xF+1],
    table0: [fn(&mut Chip8); 0xE+1],
    table8: [fn(&mut Chip8); 0xE+1],
    tableE: [fn(&mut Chip8); 0xE+1],
    tableF: [fn(&mut Chip8); 0x65+1],
    debug_mode: bool,
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

        let mut chip8: Chip8 = Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index_register: 0,
            program_counter: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 2048],
            op_code: 0,
            table: [Chip8::OP_ERR; 0xF+1],
            table0: [Chip8::OP_ERR; 0xE+1],
            table8: [Chip8::OP_ERR; 0xE+1],
            tableE: [Chip8::OP_ERR; 0xE+1],
            tableF: [Chip8::OP_ERR; 0x65+1],
            debug_mode: false,
        };

        chip8.load_fonts();
        chip8.add_table();


        return chip8;
    }

    /// Add the correct function pointer tables to the newly created Chip8 object
    pub fn add_table(&mut self) {
        if (self.debug_mode) {
            eprintln!("Loading tables");
        }
        let mut table: [fn(&mut Chip8); 0xF+1] = [Chip8::Table0, Chip8::OP_1nnn, Chip8::OP_2nnn, Chip8::OP_3xkk, Chip8::OP_4xkk, Chip8::OP_5xy0, Chip8::OP_6xkk, Chip8::OP_7xkk, Chip8::Table8, Chip8::OP_9xy0, Chip8::OP_Annn, Chip8::OP_Bnnn, Chip8::OP_Cxkk, Chip8::OP_Dxyn, Chip8::TableE, Chip8::TableF];
        // TODO: filling them with OP_ERR is redundant as I already do so when creating the array in the constructor
        let mut table0: [fn(&mut Chip8); 0xE+1] = [Chip8::OP_ERR; 0xE+1];
        let mut table8: [fn(&mut Chip8); 0xE+1] = [Chip8::OP_ERR; 0xE+1];
        let mut tableE: [fn(&mut Chip8); 0xE+1] = [Chip8::OP_ERR; 0xE+1];
        let mut tableF: [fn(&mut Chip8); 0x65+1] = [Chip8::OP_ERR; 0x65+1];

        table0[0x0] = Chip8::OP_00E0;
        table0[0xE] = Chip8::OP_00EE;

        table8[0x0] = Chip8::OP_8xy0;
        table8[0x1] = Chip8::OP_8xy1;
        table8[0x2] = Chip8::OP_8xy2;
        table8[0x3] = Chip8::OP_8xy3;
        table8[0x4] = Chip8::OP_8xy4;
        table8[0x5] = Chip8::OP_8xy5;
        table8[0x6] = Chip8::OP_8xy6;
        table8[0x7] = Chip8::OP_8xy7;
        table8[0xE] = Chip8::OP_8xyE;

        tableE[0x1] = Chip8::OP_ExA1;
        tableE[0xE] = Chip8::OP_Ex9E;

        tableF[0x07] = Chip8::OP_Fx07;
        tableF[0x0A] = Chip8::OP_Fx0A;
        tableF[0x15] = Chip8::OP_Fx15;
        tableF[0x18] = Chip8::OP_Fx18;
        tableF[0x1E] = Chip8::OP_Fx1E;
        tableF[0x29] = Chip8::OP_Fx29;
        tableF[0x33] = Chip8::OP_Fx33;
        tableF[0x55] = Chip8::OP_Fx55;
        tableF[0x65] = Chip8::OP_Fx65;


        // Apply the newly generated tables
        self.table = table;
        self.table0 = table0;
        self.table8 = table8;
        self.tableE = tableE;

        if (self.debug_mode) {
            eprintln!("Tables loaded");
        }
    }

    /// Toggle debug mode for current chip8 instance
    pub fn debug(&mut self) {
        // bitwise xor
        self.debug_mode ^= true;
        eprintln!("Debug mode activated");
    }

    /// Loads a given rom into memory, starting from memory address 0x200
    pub fn load_rom(&mut self, path: PathBuf) {
        if (self.debug_mode) {
            eprintln!("Loading ROM: {:?}", &path);
        }
        // memory addres before this are reserved
        let start_address = 0x200;

        // open rom file
        let file: File = File::open(path).expect("Error opening rom file");

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

        if (self.debug_mode) {
            eprintln!("ROM Loaded");
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

    pub fn dump_video(&mut self) -> Vec<u32> {
        let mut buf: Vec<u32> = Vec::new();

        for i in 0..self.video.len() {
            buf.push(self.video[i]);
        }
        return buf;
    }

    /// Load the fontset into memory
    fn load_fonts(&mut self) {
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
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F];
        ];

        let fontset_start_address = 0x50;

        if (self.debug_mode) {
            eprintln!("Loading fontset");
        }

        for i in 0..fontset.len() - 1 {
            self.memory[fontset_start_address + i as usize] = fontset[i as usize];
        }

        if (self.debug_mode) {
            eprintln!("Fontset loaded");
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
    #[named]
    pub fn OP_00E0(&mut self) {
        // set video buffer to zero
        self.video = [0; 2048];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 00EE - Return from subroutine
    #[named]
    fn OP_00EE(&mut self) {
        self.stack_pointer = self.stack_pointer - 1;

        self.program_counter = self.stack[self.stack_pointer as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 1NNN - Jump to location NNN(set program counter to nnn)
    #[named]
    fn OP_1nnn(&mut self) {
        // using 0x0FFF I can take the NNN from the opcode while leaving the one
        let address: u16 = self.op_code & 0x0FFF;

        self.program_counter = address;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 2NNN - Call subroutine at location NNN
    #[named]
    fn OP_2nnn(&mut self) {
        let address: u16 = self.op_code & 0x0FFF;

        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer = self.stack_pointer + 1;
        self.program_counter = address;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    // TODO: Make all of these `Vx` Variables `u8`(or maybe even usize) instead of `u16`

    /// OPCODE 3XKK - Skip next instruction if Vx = kk
    /// Since our PC has already been incremented by 2 in Cycle(), we can just increment by 2 again to skip the next instruction.
    #[named]
    fn OP_3xkk(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let byte: u16 = self.op_code & 0x00FF;

        let byte: u8 = match u8::try_from(byte) {
            Ok(number) => number,
            Err(error) => panic!(
                "Could not turn u16 into u8 in OPCODE: 3XKK. Error: {}",
                error
            ),
        };

        if self.registers[Vx as usize] == byte {
            self.program_counter += 2;
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 4XKK - Skip next instruction if Vx != kk
    #[named]
    fn OP_4xkk(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let byte: u16 = self.op_code & 0x00FF;

        let byte: u8 = match u8::try_from(byte) {
            Ok(number) => number,
            Err(error) => panic!(
                "Could not turn u16 into u8 in OPCODE: 3XKK. Error: {}",
                error
            ),
        };

        // this != is the onlh difference from the function above
        if self.registers[Vx as usize] != byte {
            self.program_counter += 2;
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 5XY0 - Skip next instruction if Vx = Vy.
    #[named]
    fn OP_5xy0(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        if self.registers[Vx as usize] == self.registers[Vy as usize] {
            self.program_counter += 2;
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 6XKK - Set Vx = kk.
    #[named]
    fn OP_6xkk(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let byte: u16 = self.op_code & 0x00FF;

        let byte: u8 = match u8::try_from(byte) {
            Ok(number) => number,
            Err(error) => panic!(
                "Could not turn u16 into u8 in OPCODE 6XKK. Error: {}",
                error
            ),
        };

        self.registers[Vx as usize] = byte;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }

    }

    /// OPCODE 7XKK - Set Vx = Vx + kk.
    #[named]
    fn OP_7xkk(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let byte: u16 = self.op_code & 0x00FF;

        let byte: u8 = match u8::try_from(byte) {
            Ok(number) => number,
            Err(error) => panic!(
                "Could not turn u16 into u8 in OPCODE 7XKK. Error: {}",
                error
            ),
        };

        self.registers[Vx as usize] += byte;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY0 - Set Vx = Vy.
    #[named]
    fn OP_8xy0(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        self.registers[Vx as usize] = self.registers[Vy as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY1 - Set Vx = Vx OR Vy.
    #[named]
    fn OP_8xy1(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        self.registers[Vx as usize] |= self.registers[Vy as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY2 - Set Vx = Vx AND Vy
    #[named]
    fn OP_8xy2(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        self.registers[Vx as usize] &= self.registers[Vy as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY3 - Set Vx = Vx XOR Vy
    #[named]
    fn OP_8xy3(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        self.registers[Vx as usize] ^= self.registers[Vy as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY4 - Set Vx = Vx + Vy, set VF = carry.
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    #[named]
    fn OP_8xy4(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        let sum: u16 = (self.registers[Vx as usize] + self.registers[Vy as usize]) as u16;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[Vx as usize] = (sum & 0xFF) as u8;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY5 - Set Vx = Vx - Vy, set VF = NOT borrow.
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    #[named]
    fn OP_8xy5(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        if self.registers[Vx as usize] > self.registers[Vy as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[Vx as usize] -= self.registers[Vy as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY6 - Set Vx = Vx SHR 1.
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    #[named]
    fn OP_8xy6(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        // Save LSB in VF
        self.registers[0xF] = self.registers[Vx as usize] & 0x1;

        self.registers[Vx as usize].shr_assign(1);
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XY7 - SUBN Vx, Vy
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    #[named]
    fn OP_8xy7(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        if self.registers[Vy as usize] > self.registers[Vx as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[Vx as usize] = self.registers[Vy as usize] - self.registers[Vx as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 8XYE - Set Vx = Vx SHL 1.
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    #[named]
    fn OP_8xyE(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        // save MSB in VF
        self.registers[0xF] = (self.registers[Vx as usize] & 0x80).checked_shr(7).unwrap_or(0);

        self.registers[Vx as usize].checked_shl(1).unwrap_or(0);
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE 9XY0 - Skip next instruction if Vx != Vy
    #[named]
    fn OP_9xy0(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy: u16 = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);

        if self.registers[Vx as usize] != self.registers[Vy as usize] {
            self.program_counter += 2;
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE ANNN - set I = nnn
    #[named]
    fn OP_Annn(&mut self) {
        let address: u16 = self.op_code & 0x0FFF;

        self.index_register = address;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE BNNN - Jump to location nnn + V0
    #[named]
    fn OP_Bnnn(&mut self) {
        let address: u16 = self.op_code & 0x0FFF;

        self.program_counter = self.registers[0] as u16 + address;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE CXKK - Set Vx = random byte AND kk.
    #[named]
    fn OP_Cxkk(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let byte: u16 = self.op_code & 0x00FF;

        let byte: u8 = match u8::try_from(byte) {
            Ok(number) => number,
            Err(error) => panic!(
                "Could not turn u16 into u8 in OPCODE CXKK. Error: {}",
                error
            ),
        };

        self.registers[Vx as usize] = self.rand_byte() & byte;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE DXYN - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    #[named]
    fn OP_Dxyn(&mut self) {
        let Vx = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let Vy = (self.op_code & 0x00F0).checked_shr(4).unwrap_or(0);
        let height = self.op_code & 0x000F;
        let VIDEO_WIDTH: u8 = 64;
        let VIDEO_HEIGHT: u8 = 32;

        // wrap if going over boundaries
        let x_pos: u8 = self.registers[Vx as usize] % VIDEO_WIDTH;
        let y_pos: u8 = self.registers[Vy as usize] % VIDEO_HEIGHT;

        self.registers[0xF] = 0;

        for row in 0..(height - 1) {
            let sprite_byte: u8 = self.memory[(self.index_register + row) as usize];

            for col in 0..(height - 1) {
                let sprite_pixel = sprite_byte & ((0x80 as u8).checked_shr(col as u32).unwrap_or(0));
                // casting without error checking here is fine because col and raw wil alwyays be lower than 255(they are 64 and 32)
                let mut screen_pixel = self.video[(((y_pos as u16 + row) * (VIDEO_WIDTH as u16) + (x_pos as u16) + col)) as usize];

                // sprite pixel is on
                if sprite_pixel != 0 {
                    // screen pixel also on - collision
                    if screen_pixel == 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }

                    // Effectively XOR with the sprite pixel
                    screen_pixel ^= 0xFFFFFFFF;
                }
            }
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE EX9E - Skip next instruction if key with the value of Vx is pressed.
    #[named]
    fn OP_Ex9E(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let key: u8 = self.registers[Vx as usize];

        if self.keypad[key as usize] != 0 {
            self.program_counter += 2;
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE EXA1 - Skip next instruction if key with the value of Vx is not pressed
    #[named]
    fn OP_ExA1(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let key: u8 = self.registers[Vx as usize];

        if self.keypad[key as usize] == 0 {
            self.program_counter += 2;
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX07 - Set Vx = delay timer value
    #[named]
    fn OP_Fx07(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        self.registers[Vx as usize] = self.delay_timer;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX0A - Wait for a key press, store the value of the key in Vx.
    #[named]
    fn OP_Fx0A(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        if self.keypad[0] != 0 {
            self.registers[Vx as usize] = 0;
        } else if self.keypad[1] != 0 {
            self.registers[Vx as usize] = 1;
        } else if self.keypad[2] != 0 {
            self.registers[Vx as usize] = 2;
        } else if self.keypad[3] != 0 {
            self.registers[Vx as usize] = 3;
        } else if self.keypad[4] != 0 {
            self.registers[Vx as usize] = 4;
        } else if self.keypad[5] != 0 {
            self.registers[Vx as usize] = 5;
        } else if self.keypad[6] != 0 {
            self.registers[Vx as usize] = 6;
        } else if self.keypad[7] != 0 {
            self.registers[Vx as usize] = 7;
        } else if self.keypad[8] != 0 {
            self.registers[Vx as usize] = 8;
        } else if self.keypad[9] != 0 {
            self.registers[Vx as usize] = 9;
        } else if self.keypad[10] != 0 {
            self.registers[Vx as usize] = 10;
        } else if self.keypad[11] != 0 {
            self.registers[Vx as usize] = 11;
        } else if self.keypad[12] != 0 {
            self.registers[Vx as usize] = 12;
        } else if self.keypad[13] != 0 {
            self.registers[Vx as usize] = 13;
        } else if self.keypad[14] != 0 {
            self.registers[Vx as usize] = 14;
        } else if self.keypad[15] != 0 {
            self.registers[Vx as usize] = 15;
        } else {
            self.program_counter -= 2;
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX15 - Set delay timer = Vx.
    #[named]
    fn OP_Fx15(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        self.delay_timer = self.registers[Vx as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX18 - Set sound timer = Vx.
    #[named]
    fn OP_Fx18(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        self.sound_timer = self.registers[Vx as usize];
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX1E - Set I = I + Vx.
    #[named]
    fn OP_Fx1E(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        self.index_register += self.registers[Vx as usize] as u16;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX29 - Set I = location of sprite for digit Vx.
    #[named]
    fn OP_Fx29(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let digit = self.registers[Vx as usize];
        let fontset_start_address = 0x50;

        self.index_register = (fontset_start_address + (5*digit)) as u16;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX33 - Store BCD representation of Vx in memory locations I, I+1, and I+2.
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    #[named]
    fn OP_Fx33(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);
        let mut value: u8 = self.registers[Vx as usize];

        // ones place
        self.memory[(self.index_register + 2) as usize] = value % 10;
        value /= 10;

        // tens place
        self.memory[(self.index_register + 1) as usize] = value % 10;
        value /= 10;

        // hundreds place
        self.memory[self.index_register as usize] = value % 10;
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX55 -- Store registers V0 to VX in memory starting at location X
    #[named]
    fn OP_Fx55(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        for i in 0..(Vx - 1) {
            self.memory[(self.index_register + i) as usize] = self.registers[i as usize];
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// OPCODE FX65 - Read registers V0 through Vx from memory starting at location I.
    #[named]
    fn OP_Fx65(&mut self) {
        let Vx: u16 = (self.op_code & 0x0F00).checked_shr(8).unwrap_or(0);

        for i in 0..(Vx - 1) {
            self.registers[i as usize] = self.memory[(self.index_register + i) as usize];
        }
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }

    /// Fallback OPCODE
    #[named]
    fn OP_ERR(&mut self) {
        eprintln!("[ERROR]: Opcode {} not valid", self.op_code); 
        if self.debug_mode {
            eprintln!("Ran opcode: {}", function_name!());
        }
    }


    #[named]
    fn Table0(&mut self) {
        if self.debug_mode {
            eprintln!("Running table: {}", function_name!());
        }
        self.table0[(self.op_code & 0x000F) as usize](self);
    }

    #[named]
    fn Table8(&mut self) {
        if self.debug_mode {
            eprintln!("Running table: {}", function_name!());
        }
        self.table8[(self.op_code & 0x000F) as usize](self);
    }

    #[named]
    fn TableE(&mut self) {
        if self.debug_mode {
            eprintln!("Running table: {}", function_name!());
        }
        self.tableE[(self.op_code & 0x000F) as usize](self);
    }

    #[named]
    fn TableF(&mut self) {
        if self.debug_mode {
            eprintln!("Running table: {}", function_name!());
        }
        self.tableF[(self.op_code & 0x00FF) as usize](self);
    }


    // the opcodes are stored in memory starting from index 512, i need to decode them and map each opcode to one of my functions
    // The CHIP-8 Architecture uses big-endian (0x00 0xe0 -> 0x00e0)

    pub fn Cycle(&mut self) {
        // Fetch opcode
        //let _: () = self.memory[self.program_counter as usize].checked_shl(8).unwrap_or(0);
        self.op_code = ((self.memory[self.program_counter as usize] as u16).checked_shl(8).unwrap_or(0)) | self.memory[(self.program_counter + 1) as usize] as u16;
        println!("Opcode: {}", &self.op_code);

        // increment pc before we do anything
        self.program_counter += 2;

        // decode and execute
        // TODO: actually implement this
        self.table[((self.op_code & 0xF000).checked_shr(12).unwrap_or(0)) as usize](self);

        // Decrement delay timer if it exists
        if (self.delay_timer > 0) {
            self.delay_timer -= 1;
        }

        // Decrement sound timer if it exists
        if (self.sound_timer > 0) {
            self.sound_timer -= 1;
        }

    }

}
