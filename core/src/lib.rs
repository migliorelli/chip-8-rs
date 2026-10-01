// Author: Miguel Migliorelli Bringhenti

use rand::random;

const START_ADDR: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const VF_ADDR: usize = 0xF;

// screen: 64x32
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

// fontset
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emulator {
    // program counter
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    keys: [bool; NUM_KEYS],

    // stack pointer
    sp: u16,
    stack: [u16; STACK_SIZE],

    // delay timer
    dt: u8,
    // sound timer
    st: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            keys: [false; NUM_KEYS],

            sp: 0,
            stack: [0; STACK_SIZE],

            dt: 0,
            st: 0,
        };
        // copy fontset into ram
        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emulator
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.keys = [false; NUM_KEYS];

        self.sp = 0;
        self.stack = [0; STACK_SIZE];

        self.dt = 0;
        self.st = 0;

        // copy fontset into ram
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // fetch opcode
        let op = self.fetch();
        // decode & execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // beep
            }

            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    fn execute(&mut self, op: u16) {
        // get opcode digits
        let d1 = (op >> 12) & 0xF;
        let d2 = (op >>  8) & 0xF;
        let d3 = (op >>  4) & 0xF;
        let d4 = op         & 0xF;

        match (d1, d2, d3, d4) {
            // NOP
            (0,0,0,0) => return,
            // CLS - Clear screen
            (0,0,0xE,0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RET - Return from subroutine
            (0,0,0xE,0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            // JMP - Jump NNN
            (1,_,_,_) => {
                let nnn = op & 0x0FFF;
                self.pc = nnn;
            },
            // JSR - Jump to subroutine NNN
            (2,_,_,_) => {
                let nnn = op & 0x0FFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            // SKE - Skip equal (if VX == NN)
            (3,_,_,_) => {
                let x = d2 as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            // SNE - Skip not equal (if VX != NN)
            (4,_,_,_) => {
                let x = d2 as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            // SKR - Skip if registers equal (if VX == VY)
            (5,_,_,0) => {
                let x = d2 as usize;
                let y = d3 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // SET - Set register value
            (6,_,_,_) => {
                let x = d2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = nn;
            },
            // ADD - Add without carry
            (7,_,_,_) => {
                let x = d2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            // CPY - Copy VY into VX
            (8,_,_,0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            // ORR - Bitwise OR
            (8,_,_,1) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            // AND - Bitwise AND
            (8,_,_,2) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            // XOR - Bitwise XOR
            (8,_,_,3) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            // ADC - Add with carry (VF = 1 if it overflows)
            (8,_,_,4) => {
                let x = d2 as usize;
                let y = d3 as usize;
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[VF_ADDR] = new_vf;
            },
            // SUB - Subtract (VF = 0 if it borrows)
            (8,_,_,5) => {
                let x = d2 as usize;
                let y = d3 as usize;
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[VF_ADDR] = new_vf;
            }
            // SHR - Shift right (VF = LSB)
            (8,_,_,6) => {
                let x = d2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[VF_ADDR] = lsb;
            },
            // RSB - Reverse subtract (VF = 0 if it borrows)
            (8,_,_,7) => {
                let x = d2 as usize;
                let y = d3 as usize;
                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[VF_ADDR] = new_vf;
            }
            // SHL - Shift left (VF = MSB)
            (8,_,_,0xE) => {
                let x = d2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[VF_ADDR] = msb;
            },
            // SNR - Skip if not equal registers (VX != VY)
            (9,_,_,0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // LDI - Load index (I Register = NNN)
            (0xA,_,_,_) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            },
            // JPA - Jump with address offset (jump to NNN + V0)
            (0xB,_,_,_) => {
                let nnn = op & 0x0FFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            // RND - Generate a pseudorandom byte with a mask (VX = rand() & NN)
            (0xC,_,_,_) => {
                let x = d2 as usize;
                let nn = (op & 0x00FF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            // DRW - Draw sprite (width: 8; height: N; x: d2; y: d3)
            // VF = flipped ? 1 : 0
            (0xD,_,_,_) => {
                // get (x,y) coords
                let x_coord = self.v_reg[d2 as usize] as u16;
                let y_coord = self.v_reg[d3 as usize] as u16;

                // the last digit corresponds to the number of rows of the sprite
                let num_rows = d4;
                // keep track if any pixels were flipped
                let mut flipped = false;

                for y_line in 0..num_rows {
                    // determine row's data memory address
                    let addr = self.i_reg + y_line;
                    let pixels = self.ram[addr as usize];

                    // iterate over each column
                    for x_line in 0..8 {
                        // use a mask to fetch current pixel's bit.
                        // flip if it equals 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // get pixel's index
                            let idx = x + SCREEN_WIDTH * y;
                            // flip the pixel
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                // change VF register's value
                if flipped {
                    self.v_reg[VF_ADDR] = 1;
                } else {
                    self.v_reg[VF_ADDR] = 0;
                }
            },
            // SKP - Skip if key (VX) is pressed
            (0xE,_,9,0xE) => {
                let x = d2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // SKN - Skip if key (VX) is not pressed
            (0xE,_,0xA,1) => {
                let x = d2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            // GDT - Get delay timer (VX = DT)
            (0xF,_,0,7) => {
                let x = d2 as usize;
                self.v_reg[x] = self.dt;
            },
            // WKP - Wait for any key press and saves into VX
            (0xF,_,0,0xA) => {
                let x = d2 as usize;
                let mut pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                
                if !pressed {
                    self.pc -= 2;
                }
            },
            // STD - Set delay timer (DT = VX)
            (0xF,_,1,5) => {
                let x = d2 as usize;
                self.dt = self.v_reg[x];
            // SST - Set sound timer (ST = VX)
            }
            (0xF,_,1,8) => {
                let x = d2 as usize;
                self.st = self.v_reg[x];
            }
            // ADI - Add to index (I Register += VX)
            (0xF,_,1,0xE) => {
                let x = d2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            // FNT - Load font sprite (I Register = VX)
            (0xF,_,2,9) => {
                let x = d2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }
            // BCD - Binary-coded decimal (converts VX to decimal)
            (0xF,_,3,3) => {
                let x = d2 as usize;
                let vx = self.v_reg[x] as f32;
                // fetch hundreds digit
                let hundreds = (vx / 100.0).floor() as u8;
                // fetch tens digit
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // fetch one digit
                let ones = (vx % 10.0) as u8;

                // stores on ram sequentially (i+0,i+1,i+2)
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            // STR - Store registers (Saves V0 .. VX into ram from I Register's location)
            (0xF,_,5,5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }
            // LDR - Load registers (Loads V0 .. VX from ram from I Register's location)
            (0xF,_,6,5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }
            (_,_,_,_) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}

