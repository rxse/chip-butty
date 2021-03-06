#![allow(dead_code)]
extern crate rand;

use std::io::prelude::*;
use std::io::Result as IOResult;
use std::io::Error as IOError;
use std::io::ErrorKind;
use std::fs::File;
use std::path::Path;
use self::rand::Rng;
use self::rand::thread_rng;

pub type Rom = Vec<u8>;
pub type Opcode = u16;

pub struct Chip8 {
    // V0 to VF, each one byte.
    pub registers: [u8; 16],
    // Arbitrary sized as the stack is not
    // accessed manually.
    pub stack:     Vec<usize>,
    // 0x1000 bytes of addressable memory.
    pub memory:    [u8; 0x1000],
    // Address register, I.
    pub index:     u16,
    // Program counter.
    pub counter:   usize,
    // Delay timer.
    pub delay:     u8,
    // Sound timer.
    pub sound:     u8,
    // Screen
    pub screen: [[bool; 64]; 32],
    // Something that implements Render for screen drawing.
    // Or, no screen.
    pub renderer: Option<Box<Render>>
}

pub trait Render {
    fn clear(&self, screen: &mut [[bool; 64]; 32]);
}

trait Parameters {
    fn nnn(&self) -> u16;
    fn nn(&self) -> u8;
    fn n(&self) -> u8;
    fn x(&self) -> u8;
    fn y(&self) -> u8;
}

impl Parameters for Opcode {
    // Address.
    fn nnn(&self) -> u16 {
        self & 0x0FFF
    }

    // 8 bit constant.
    fn nn(&self) -> u8 {
        (self & 0x00FF) as u8
    }

    // 4 bit constant.
    fn n(&self) -> u8 {
        (self & 0x000F) as u8
    }

    // 4 bit register identifiers.
    fn x(&self) -> u8 {
        ((self & 0x0F00) >> 8) as u8
    }

    fn y(&self) -> u8 {
        ((self & 0x00F0) >> 4) as u8
    }
}

impl Chip8 {
    pub fn new(renderer: Option<Box<Render>>) -> Chip8 {
        Chip8 {
            registers: [0; 16],
            stack: vec![],
            memory: [0; 0x1000],
            index: 0,
            counter: 0x200,
            delay: 0,
            sound: 0,
            screen: [[false; 64]; 32],
            renderer: renderer
        }
    }
    
    pub fn emulate(&mut self, op: Opcode) {
        macro_rules! not_implemented {
            () => {
                println!("{:#X} is not implemented!", op)
            }
        }

        // Macro for reading a register.
        // It converts the index to the Index-compatible usize.
        macro_rules! register {
            ($reg:expr) => {
                self.registers[$reg as usize]
            }
        }
        
        match op & 0xF000 {
            0x0000 => {
                // Clears the screen.
                if op == 0x00E0 {
                    if let Some(ref renderer) = self.renderer {
                        renderer.clear(&mut self.screen)
                    }
                }
                
                // Returns from a subroutine.
                else if op == 0x00EE {
                    self.counter = self.stack.pop().unwrap()
                }
                
                // Calls RCA 1802 program at the address.
                else { not_implemented!() }
            },

            // Jumps to address.
            0x1000 => {
                self.counter = op.nnn() as usize
            },

            // Calls subroutine at address.
            0x2000 => {
                self.stack.push(self.counter);
                self.counter = op.nnn() as usize
            },

            // Skips the next instruction
            // if VX equals NN.
            0x3000 => {
                if register!(op.x()) == op.nn() {
                    self.counter += 2
                }
            },

            // Skips the next instruction
            // if VX doesn't equal NN.
            0x4000 => {
                if register!(op.x()) != op.nn() {
                    self.counter += 2
                }
            },

            // Skips the next instruction
            // if VX equals VY.
            0x5000 => {
                if register!(op.x()) == register!(op.y()) {
                    self.counter += 2
                }
            },

            // Sets VX to NN.
            0x6000 => {
                let vx = op.x();
                register!(vx) = op.nn()
            },

            // Adds NN to VX.
            0x7000 => {
                let vx = op.x();
                let new = {
                    register!(vx).wrapping_add(op.nn())
                };
                
                register!(vx) = new
            },

            0x8000 => {
                let mode = op.n();

                if mode == 0x0 {
                    let vy = register!(op.y());
                    register!(op.x()) = vy;
                }

                else if mode == 0x1 {
                    let vx = register!(op.x());
                    let vy = register!(op.y());
                    register!(op.x()) = vx | vy;
                }

                else if mode == 0x2 {
                    let vx = register!(op.x());
                    let vy = register!(op.y());
                    register!(op.x()) = vx & vy;
                }

                else if mode == 0x3 {
                    let vx = register!(op.x());
                    let vy = register!(op.y());
                    register!(op.x()) = vx ^ vy;
                }

                else if mode == 0x4 {

                }

                else { not_implemented!() }
            },

            // Skips the next instruction
            // if VX doesn't equal VY.
            0x9000 => {
                not_implemented!()
            },

            // Sets I to the address NNN.
            0xA000 => {
                self.index = op.nnn()
            },

            // Jumps to the address NNN plus V0.
            0xB000 => {
                let address = op.nnn() + (register!(0) as u16);
                self.counter = address as usize
            },

            // Sets VX to the result of a bitwise
            // AND operation on a random number and NN.
            0xC000 => {
                let rn = thread_rng().gen::<u8>();
                register!(op.x()) = rn & op.nn() 
            },

            // Weird sprite stuff.
            0xD000 => {
                not_implemented!()
            },

            0xE000 => {
                not_implemented!()
            },

            0xF000 => {
                let mode = op.nn();
                
                if mode == 0x07 {
                    register!(op.x()) = self.delay
                }

                else if mode == 0x0A {
                    not_implemented!()
                }

                else if mode == 0x15 {
                    self.delay = op.x()
                }

                else if mode == 0x18 {
                    self.sound = op.x()
                }

                else if mode == 0x1E {
                    self.index += register!(op.x()) as u16
                }

                else if mode == 0x29 {
                    not_implemented!()
                }

                else if mode == 0x33 {
                    not_implemented!()
                }

                else if mode == 0x55 {
                    let register = op.x();                    
                    
                    for i in 0 .. (register + 1) {
                        let pos = (self.index as usize) + i as usize;
                        self.memory[pos] = register!(i)
                    }
                }

                else if mode == 0x65 {
                    let register = op.x();

                    for i in 0 .. (register + 1) {
                        let pos = (self.index as usize) + i as usize;
                        register!(i) = self.memory[pos]
                    }
                }

                else { not_implemented!() }
            },
            
            _ => { not_implemented!() }
        }
    }

    /// Read a file into program memory.
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> IOResult<()> {
        let mut program: Vec<u8> = vec![];
        let mut file = try!(File::open(path));

        // Return with an error if there's no space.
        if try!(file.read_to_end(&mut program)) > (0x1000 - 200) {
            Err(IOError::new(ErrorKind::Other, "ROM is too large!"))
        }

        else {
            let region = &mut self.memory[0x200..(0x200 + program.len())];
            region.clone_from_slice(&program);
            Ok(())
        }
    }

    /// Run the program contained in memory.
    /// This function will never return.
    pub fn run(&mut self) -> ! {        
        loop {
            let op = {
                let p1 = (self.memory[self.counter] as u16) << 8;
                let p2 = self.memory[self.counter + 1] as u16;
                p1 + p2
            };
            
            self.emulate(op);
            self.counter += 2;
        }
    }
}
