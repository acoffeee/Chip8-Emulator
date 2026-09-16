const KEY_COUNT: usize = 16;
const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_LEVELS: usize = 16;

const VIDEO_HEIGHT: usize = 32;
const VIDEO_WIDTH: usize = 64;

const START_ADDRESS: u16 = 0x200;
const FONTSET_SIZE: usize = 80;
const FONTSET_START_ADDRESS: u16 = 0x50;
const SCREEN_WIDTH: u8 = 128;
const SCREEN_HEIGHT: u8 = 128;

struct Chip8 {
    registers: [u8; REGISTER_COUNT],
    memory: [u8; MEMORY_SIZE],
    index: u16,
    pc: u16,
    stack: [u16; STACK_LEVELS],
    sp: u8,
    opcode: u16,
    delayTimer: u8,
    soundTimer: u8,
    keypad: [u8; KEY_COUNT],
    video: [u32; VIDEO_WIDTH * VIDEO_HEIGHT],
}

const fontset: [u8; 5 * 16] = [
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

// NOTE : opcode & 0x0FFF is throwing away the first nibble (4bits) (the instruction) and keeping the following 3 nibbles (12 bits)
impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            registers: [0; REGISTER_COUNT],
            memory: [0; MEMORY_SIZE],
            index: 0,
            pc: START_ADDRESS, //start adress, something about the previous being reserved for metadata io
            stack: [0; STACK_LEVELS],
            sp: 0,
            opcode: 0,
            delayTimer: 0,
            soundTimer: 0,
            keypad: [0; KEY_COUNT],
            video: [0; VIDEO_WIDTH * VIDEO_HEIGHT],
        }
    }
    fn load_array(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.memory[(START_ADDRESS + i as u16) as usize] = data[i];
        }
    }
    /**
    fn load_ROM()
    **/

    ///Operator: CLS
    /// Suposed to clear the display
    /// It would just 0 out the video buffer
    fn OP_00E0(&mut self) {
        self.video = [0; 64 * 32]; //litterally just realloc it
    }
    /// Operator: RET
    /// decrements the stack pointer, and sets the program counter to that element, so that the next instruction was the instruction below that on the stack
    fn OP_00EE(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }
    ///Operator: JP addr
    ///jumps to location nnn
    ///unlike a call, which would put the next instruction on the call stack, it doesnt remember its possition
    fn OP_1nnn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;
        self.pc = addr;
    }
    ///Operator: CALL addr
    /// puts the next instruction/current pg on the call stack
    ///import to put next, other wise it would infinitely call
    fn OP_2nnn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;
        self.stack[self.sp as usize] = self.pc; //push next instruction onto stack
        self.sp += 1;
        self.pc = addr;
    }
    ///Operator: SE Vx, byte
    ///skip next instruction if register[Vx as usize]== kk
    fn OP_3xkk(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        if self.registers[Vx as usize] == byte {
            self.pc += 2;
        }
    }
    ///Operator: SNE Vx, byte
    ///skip next instruction if register[Vx as usize]!= kk
    fn OP_4xkk(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        if self.registers[Vx as usize] != byte {
            self.pc += 2;
        }
    }
    ///Operator: SE Vx, Vy
    ///skip next instruction if register[Vx as usize]== register[Vy]
    fn OP_5xy0(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        if self.registers[Vx as usize] == self.registers[Vy as usize] {
            self.pc += 2;
        }
    }
    ///Operator: LD Vx, byte
    /// register[Vx as usize]= byte
    fn OP_6xkk(&mut self) {
        let Vx = self.opcode & 0x0F00 >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        self.registers[Vx as usize] = byte;
    }
    ///Operator: ADD Vx, byte
    /// register[Vx as usize]+= byte
    fn OP_7xkk(&mut self) {
        let Vx = self.opcode & 0x0F00 >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        self.registers[Vx as usize] += byte;
    }
    ///Operator: LD Vx, Vy
    /// register[Vx as usize]= register[Vy]
    fn OP_8xy0(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] = self.registers[Vy as usize];
    }
    ///Operator: OR Vx, Vy
    /// register[Vx as usize]= register[Vx as usize]or register[Vy]
    fn OP_8xy1(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] |= self.registers[Vy as usize];
    }
    ///Operator: AND Vx, Vy
    /// register[Vx as usize]= register[Vx as usize]XOR register[Vy]
    fn OP_8xy2(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] ^= self.registers[Vy as usize];
    }
    ///Operator: XOR Vx, Vy
    /// register[Vx as usize]= register[Vx as usize]or register[Vy]
    fn OP_8xy3(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] |= self.registers[Vy as usize];
    }
    ///Operator: AND Vx, Vy
    /// register[Vx as usize]= register[Vx as usize]+ register[Vy], set VF = carry
    /// if register[Vx as usize]+ register[Vy] > 8bits, VF = 1 other wise 0. only lowest 8 bits are kept and stored in Vx
    ///TL;DR Add w/ overflow flag
    fn OP_8xy4(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        let sum = (self.registers[Vx as usize] + self.registers[Vy as usize]) as u16;
        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[Vx as usize] = (sum & 0xFF) as u8;
    }
    ///Operator: SUB Vx, Vy
    /// Vx = Vx - Vy, set VF = not borrow
    /// if Vx > Vy, then VF is set to 1, other wise 0. Then Vy is subtracted from Vx and the result is stored in Vx.
    fn OP_8xy5(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        if self.registers[Vx as usize] > self.registers[Vy as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[Vx as usize] -= self.registers[Vy as usize];
    }
    ///Operator: SHR Vx
    /// Vx = Vx shifted to the right 1 bit
    ///if least significant bit is 1, then set vf to 1 other wise 0.
    fn OP_8xy6(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        self.registers[0xF] = (self.registers[Vx as usize] & 0x1); // return the first bit
        self.registers[Vx as usize] >>= 1;
    }
    ///Operator: SUBN Vx, Vy
    /// Vx = Vy - Vx, set VF = not borrow
    /// if Vy > Vx, then VF is set to 1, other wise 0. Then Vx is subtracted from Vy and the result is stored in Vx.
    fn OP_8xy7(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        if self.registers[Vy as usize] > self.registers[Vx as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[Vx as usize] = self.registers[Vy as usize] - self.registers[Vx as usize];
    }
    ///Operator: SHL Vx {, Vy}
    //tbh idrk what Vy is doing but it was in my reference so might as well document it
    ///if most significant bit is 1 then set vf = 1 else 0, Vx is multiplied by 2
    fn OP_8xyE(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        self.registers[0xF] = (self.registers[Vx as usize] & 0x80); // return the last bit
        self.registers[Vx as usize] <<= 1;
    }
    ///Operator: SNE Vxm, Vy
    ///skip next instruction if Vx != Vy
    fn OP_9xy0(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        if self.registers[Vx as usize] != self.registers[Vy as usize] {
            self.pc += 2;
        }
    }
    ///Operator: LD I, addr
    ///set I = nnn (nibblenibblenibble)
    fn OP_Annn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;
        self.index = addr;
    }
    ///Operator: JP V0, addr
    /// jump to location addr/nnn + register[0]
    fn OP_Bnnn(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.pc = self.registers[0] as u16 + addr;
    }
    ///Operator: RND Vx, byte
    /// Vx = random byte and kk
    fn OP_CxKK(&mut self) {
        let Vx = (self.opcode & 0x0F00);
        let byte = (self.opcode & 0x00FF) as u8;
        let rand_byte: u8 = rand::random();
        self.registers[Vx as usize] = rand_byte & byte;
    }
    ///Operator: DRW Vx, Vy, nibble
    ///draws a sprite starting at I(Vx,Vy), set VF = collision
    ///if while drawing, we come across a pixel that already has a sprite pixel, that is collision. If our pixel in that location is set, VF = true to express that.
    ///then we just xor the pixeel with 0xFFFFFFFF too xor it with the sprite pixel (which we now know is on).
    ///We cant xor it directly because the sprite pixel is either 1 or 0 while our video pixel is either 0x00000000 or 0xFFFFFFFF
    fn OP_Dxyn(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        let height = self.opcode & 0x000F;

        let xPos = self.registers[Vx as usize] % SCREEN_WIDTH; //width
        let yPos = self.registers[Vy as usize] & SCREEN_HEIGHT; // height
        self.registers[0xF] = 0;

        for row in 0..SCREEN_HEIGHT {
            let row = row as u16;
            let spriteByte = self.memory[(self.index + row) as usize];
            for col in 0..8 {
                let spritePixel = spriteByte & (0x80 >> col); //extract column from byte
                let screenPixel =
                    &mut self.video[((yPos + row as u8) * SCREEN_WIDTH + (xPos + col)) as usize];

                //Sprite Pixel is on
                if spritePixel > 0 {
                    //if colision
                    if *screenPixel > 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }

                    *screenPixel ^= 0xFFFFFFFF;
                }
            }
        }
    }
    ///Operator: SKP Vx
    ///skip next instruction if key withg value of Vx is pressed
    fn OP_Ex9E(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[Vx as usize];
        if self.keypad[key as usize] > 0 {
            self.pc += 2;
        }
    }
    ///Operator: SKNP Vx
    ///skip next instruction if key with value of Vx is not pressed
    fn OP_ExA1(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[Vx as usize];
        if self.keypad[key as usize] == 0 {
            self.pc += 2;
        }
    }
    ///Operator: LD Vx, DT
    ///Vx = delay timer value
    fn OP_Fx07(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.registers[Vx as usize] = self.delayTimer;
    }
    ///Operator: LD Vx, K
    ///wait for a key press then store the vvalue of a keyu in Vx
    ///easiest way to wait is pc -= 2 whjich basically just causes the pc to stay stagnent and hence run this command until a key stroke is made
    fn OP_Fx0A(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        //iters thro array and returns first index where there is a key, if no keys then none is returned and we loop this statement,
        match self.keypad.iter().position(|&k| k != 0) {
            Some(key) => self.registers[Vx as usize] = key as u8,
            None => self.pc -= 2,
        }
    }
    ///Operator: LD DT, Vx
    /// set the delay timer = Vx
    fn OP_Fx15(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.delayTimer = self.registers[Vx as usize];
    }
    ///Operator: LD ST, Vx
    ///sound timer = Vx
    fn OP_Fx18(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.soundTimer = self.registers[Vx as usize];
    }
    ///Operator ADD I, Vx
    /// Index = Index + Vx
    fn OP_Fx1E(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.index += self.registers[Vx as usize] as u16;
    }
    ///Operator: LD F, Vx
    /// set I = location of spirte for digit at register[Vx]
    /// since font characters are at 0x50, and a font char is 5 byte each so we can get the addr of the first byte by just taking the offset from the start adress
    fn OP_Fx29(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let digit = self.registers[Vx as usize];
        self.index = FONTSET_START_ADDRESS + (5 * digit) as u16
    }
    ///Operator: LD, B, Vx
    ///Store BCD of Vx in I, I+1, I+2
    fn OP_Fx33(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let mut value = self.registers[Vx as usize];
        //ones place
        self.memory[(self.index + 2) as usize] = value % 10;
        value /= 10;
        //tens place
        self.memory[(self.index + 1) as usize] = value % 10;
        value /= 10;
        //one hundres place
        self.memory[self.index as usize] = value % 10;
    }
    ///Operator: LD [I], Vx
    ///stores registers V0-Vx in memory starting at location I in memory
    fn OP_Fx55(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        for i in 0..=Vx {
            self.memory[(self.index + i) as usize] = self.registers[i as usize];
        }
    }
    ///Operator: ld Vx, [i]
    ///writes into registers V0-Vx from memory starting at [i]
    fn OP_Fx65(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        for i in 0..=Vx {
            self.registers[i as usize] = self.memory[(self.index + i) as usize];
        }
    }
    ///Does nothing but is technically an opcode someone could pass
    fn OP_NULL(&self) {
        println!("Invalid OP code!: {}", self.opcode);
    }
    fn Cycle(&mut self) {
        //fetch memory and memory +1 cause op code is split into two bytes
        //Fetch first 8 bits, shift to the left 8 bits of the 16 bit opcode, then or the bottom bits to essentially combind them.
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;
        //memory location of next opcode
        self.pc += 2;
        let first_nibble = self.opcode >> 12 as u8;
        let third_nibble = (self.opcode << 8) >> 12 as u8;
        let last_nibble = (self.opcode << 12) >> 12 as u8;
        match (first_nibble) {
            0x0 => match (last_nibble) {
                0x0 => self.OP_00E0(),
                0xE => self.OP_00EE(),
                _ => self.OP_NULL(),
            },
            0x1 => self.OP_1nnn(),
            0x2 => self.OP_2nnn(),
            0x3 => self.OP_3xkk(),
            0x4 => self.OP_4xkk(),
            0x5 => self.OP_5xy0(),
            0x6 => self.OP_6xkk(),
            0x7 => self.OP_7xkk(),
            0x8 => match (last_nibble) {
                0x0 => self.OP_8xy0(),
                0x1 => self.OP_8xy1(),
                0x2 => self.OP_8xy2(),
                0x3 => self.OP_8xy3(),
                0x4 => self.OP_8xy4(),
                0x5 => self.OP_8xy5(),
                0x6 => self.OP_8xy6(),
                0x7 => self.OP_8xy7(),
                0xE => self.OP_8xyE(),
                _ => self.OP_NULL(),
            },
            0x9 => self.OP_9xy0(),
            0xA => self.OP_Annn(),
            0xB => self.OP_Bnnn(),
            0xC => self.OP_CxKK(),
            0xD => self.OP_Dxyn(),
            0xE => match (last_nibble) {
                0x1 => self.OP_ExA1(),
                0xE => self.OP_Ex9E(),
                _ => self.OP_NULL(),
            },
            0xF => match (last_nibble) {
                0x07 => self.OP_Fx07(),
                0x0A => self.OP_Fx0A(),
                0x15 => self.OP_Fx15(),
                0x18 => self.OP_Fx18(),
                0x1E => self.OP_Fx1E(),
                0x29 => self.OP_Fx29(),
                0x33 => self.OP_Fx33(),
                0x55 => self.OP_Fx55(),
                0x65 => self.OP_Fx65(),
                _ => self.OP_NULL(),
            },
            _ => self.OP_NULL(),
        }
        if self.delayTimer > 0 {
            self.delayTimer -= 1;
        }
        if self.soundTimer > 0 {
            self.soundTimer -= 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
