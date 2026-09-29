use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
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
    port_tx: Option<Sender<u8>>,
    port_rx: Option<Receiver<u8>>,
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
    fn new(sender: Option<Sender<u8>>, recv: Option<Receiver<u8>>) -> Chip8 {
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
            port_tx: sender,
            port_rx: recv,
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

    ///Operator: cls
    /// Suposed to clear the display
    /// It would just 0 out the video buffer
    fn OP_00E0(&mut self) {
        self.video = [0; 64 * 32]; //litterally just realloc it
    }
    /// Operator: ret
    /// decrements the stack pointer, and sets the program counter to that element, so that the next instruction was the instruction below that on the stack
    fn OP_00EE(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }
    ///Operator: jp addr
    ///jumps to location nnn
    ///unlike a call, which would put the next instruction on the call stack, it doesnt remember its possition
    fn OP_1nnn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;
        self.pc = addr;
    }
    ///Operator: call addr
    /// puts the next instruction/current pg on the call stack
    ///import to put next, other wise it would infinitely call
    fn OP_2nnn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;
        self.stack[self.sp as usize] = self.pc; //push next instruction onto stack
        self.sp += 1;
        self.pc = addr;
    }
    ///Operator: se Vx, byte
    ///skip next instruction if register[Vx as usize]== kk
    fn OP_3xkk(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        if self.registers[Vx as usize] == byte {
            self.pc += 2;
        }
    }
    ///Operator: sne Vx, byte
    ///skip next instruction if register[Vx as usize]!= kk
    fn OP_4xkk(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        if self.registers[Vx as usize] != byte {
            self.pc += 2;
        }
    }
    ///Operator: se Vx, Vy
    ///skip next instruction if register[Vx as usize]== register[Vy]
    fn OP_5xy0(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        if self.registers[Vx as usize] == self.registers[Vy as usize] {
            self.pc += 2;
        }
    }
    ///Operator: ld Vx, byte
    /// register[Vx as usize]= byte
    fn OP_6xkk(&mut self) {
        let Vx = self.opcode & 0x0F00 >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        self.registers[Vx as usize] = byte;
    }
    ///Operator: add Vx, byte
    /// register[Vx as usize]+= byte
    fn OP_7xkk(&mut self) {
        let Vx = self.opcode & 0x0F00 >> 8; //register number
        let byte = (self.opcode & 0x00FF) as u8; //actual data
        self.registers[Vx as usize] += byte;
    }
    ///Operator: ld Vx, Vy
    /// register[Vx as usize]= register[Vy]
    fn OP_8xy0(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] = self.registers[Vy as usize];
    }
    ///Operator: or Vx, Vy
    /// register[Vx as usize]= register[Vx as usize]or register[Vy]
    fn OP_8xy1(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] |= self.registers[Vy as usize];
    }
    ///Operator: and Vx, Vy
    /// register[Vx as usize]= register[Vx as usize]XOR register[Vy]
    fn OP_8xy2(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] ^= self.registers[Vy as usize];
    }
    ///Operator: xor Vx, Vy
    /// register[Vx as usize]= register[Vx as usize]or register[Vy]
    fn OP_8xy3(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 4; //register number
        self.registers[Vx as usize] |= self.registers[Vy as usize];
    }
    ///Operator: and Vx, Vy
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
    ///Operator: sub Vx, Vy
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
    ///Operator: shr Vx
    /// Vx = Vx shifted to the right 1 bit
    ///if least significant bit is 1, then set vf to 1 other wise 0.
    fn OP_8xy6(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        self.registers[0xF] = (self.registers[Vx as usize] & 0x1); // return the first bit
        self.registers[Vx as usize] >>= 1;
    }
    ///Operator: subn Vx, Vy
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
    ///Operator: shl Vx {, Vy}
    //tbh idrk what Vy is doing but it was in my reference so might as well document it
    ///if most significant bit is 1 then set vf = 1 else 0, Vx is multiplied by 2
    fn OP_8xyE(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        self.registers[0xF] = (self.registers[Vx as usize] & 0x80); // return the last bit
        self.registers[Vx as usize] <<= 1;
    }
    ///Operator: sne Vxm, Vy
    ///skip next instruction if Vx != Vy
    fn OP_9xy0(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        if self.registers[Vx as usize] != self.registers[Vy as usize] {
            self.pc += 2;
        }
    }
    ///Operator: ld I, addr
    ///set I = nnn (nibblenibblenibble)
    fn OP_Annn(&mut self) {
        let addr: u16 = self.opcode & 0x0FFF;
        self.index = addr;
    }
    ///Operator: jp V0, addr
    /// jump to location addr/nnn + register[0]
    fn OP_Bnnn(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.pc = self.registers[0] as u16 + addr;
    }
    ///Operator: rnd Vx, byte
    /// Vx = random byte and kk
    fn OP_CxKK(&mut self) {
        let Vx = (self.opcode & 0x0F00);
        let byte = (self.opcode & 0x00FF) as u8;
        let rand_byte: u8 = rand::random();
        self.registers[Vx as usize] = rand_byte & byte;
    }
    ///Operator: drw Vx, Vy, nibble
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
    ///Operator: skp Vx
    ///skip next instruction if key withg value of Vx is pressed
    fn OP_Ex9E(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[Vx as usize];
        if self.keypad[key as usize] > 0 {
            self.pc += 2;
        }
    }
    ///Operator: sknp Vx
    ///skip next instruction if key with value of Vx is not pressed
    fn OP_ExA1(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[Vx as usize];
        if self.keypad[key as usize] == 0 {
            self.pc += 2;
        }
    }
    ///Operator: ld Vx, DT
    ///Vx = delay timer value
    fn OP_Fx07(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.registers[Vx as usize] = self.delayTimer;
    }
    ///Operator: ld Vx, K
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
    ///Operator: delay DT, Vx
    /// set the delay timer = Vx
    fn OP_Fx15(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.delayTimer = self.registers[Vx as usize];
    }
    ///Operator: sound ST, Vx
    ///sound timer = Vx
    fn OP_Fx18(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.soundTimer = self.registers[Vx as usize];
    }
    ///Operator add I, Vx
    /// Index = Index + Vx
    fn OP_Fx1E(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.index += self.registers[Vx as usize] as u16;
    }
    ///Operator: hex F, Vx
    /// set I = location of spirte for digit at register[Vx]
    /// since font characters are at 0x50, and a font char is 5 byte each so we can get the addr of the first byte by just taking the offset from the start adress
    fn OP_Fx29(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let digit = self.registers[Vx as usize];
        self.index = FONTSET_START_ADDRESS + (5 * digit) as u16
    }
    ///Operator: bcd, B, Vx
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
    ///Operator: stor [I], Vx
    ///stores registers V0-Vx in memory starting at location I in memory
    fn OP_Fx55(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        for i in 0..=Vx {
            self.memory[(self.index + i) as usize] = self.registers[i as usize];
        }
    }
    ///Operator: rstr Vx, [i]
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
    //chip8e extension. reference found hjere https://chip-8.github.io/extensions/#chip-8e
    ///implementation guide found here https://github.com/trapexit/chip-8_documentation/blob/master/VIPER/VIPER_-_Volume_2_-_Issue_08_09.pdf
    //i will make the nmenics since they didnt

    ///Operator: stop
    ///Traps cpu in infinite loop, effectively killing execution.
    fn OP_00ED(&mut self) {
        self.pc -= 2
    }
    ///Operator: wait
    /// this will loop until the timer is up
    fn OP_0151(&mut self) {
        if self.delayTimer != 0 {
            self.pc -= 2;
        }
    }
    ///Operator: nope
    /// does nothing i guess
    fn OP_00F2(&self) {
        ()
    }
    ///Operation: sp
    ///skips the next instruction
    fn OP_0188(&mut self) {
        self.pc += 2;
    }
    ///Operator: spg Vx, Vy
    /// Skips next instruction if Vx > Vy
    fn OP_5XY1(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8; //register number
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        if self.registers[Vx as usize] > self.registers[Vy as usize] {
            self.pc += 2;
        }
    }
    ///Operator: stor Vx, Vy
    /// transfers registers X -> y to memory
    fn OP_5XY2(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        for i in Vx..=Vy {
            self.memory[(self.index + i) as usize] = self.registers[i as usize];
        }
    }
    ///Operator: rstr Vx, Vy
    /// transfers memory X -> y to registers
    fn OP_5XY3(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        let Vy = (self.opcode & 0x00F0) >> 8; //register number
        for i in Vx..=Vy {
            self.registers[i as usize] = self.memory[(self.index + i) as usize];
        }
    }
    ///Operator: jpb NN
    /// jumps to current memory location - NN
    fn OP_BBNN(&mut self) {
        let NN = self.opcode & 0xFF;
        self.pc -= NN - 2;
    }
    ///Operator: jpf NN
    /// jumps to current memory location + NN
    fn OP_BFNN(&mut self) {
        let NN = self.opcode & 0xFF;
        self.pc += NN - 2;
    }
    ///Operator: out VX
    ///ouputs the content of VX to output port 3
    fn OP_FX03(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.port_tx
            .as_mut()
            .expect("NOT IMPLEMENTED")
            .send(self.registers[Vx as usize])
            .unwrap();
    }
    ///Operator: sp VX
    /// skips Vx amount of bytes. if vx is 0, do nothing
    ///supposed to substitute for BNNN
    fn OP_FX1B(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        if (Vx == 0) {
            return;
        }
        self.pc += self.registers[Vx as usize] as u16 - 2;
    }
    //note that for these next instructions that use port 3, i implemented it with threading. basically instead of ports we got the rx and tx channel. although its not perfect, this project lowkey already cooked.
    ///Operator: hlt VX
    ///sets timer to VX then waits at 0151 until timer is 0151
    fn OP_FX4F(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.delayTimer = self.registers[Vx as usize];
        loop {
            if self.delayTimer == 0 {
                return;
            }
            thread::sleep(std::time::Duration::from_secs(1));
            self.delayTimer -= 1;
        }
    }
    ///Operator: hltread VX
    ///will wait for any content on the rx channel, then read it into vx
    // this WILL HANG IF NO DATA IS SENT so make sure to use it well.
    fn OP_FXE3(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.registers[Vx as usize] = match self.port_rx.as_ref().unwrap().recv() {
            Ok(data) => data,
            Err(_) => 0,
        };
    }
    ///Operator: read VX
    ///attempts to read from rx without waiting
    /// if no data is thgere or it other wise fails it propogates with 0
    fn OP_FXE7(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.registers[Vx as usize] = match &self.port_rx.as_ref().unwrap().try_recv() {
            Ok(data) => *data,
            Err(_) => 0,
        };
    }
    //this is the start of the chip-8C(offeee) extension

    ///Operator: stor Vx
    /// stores register vx at i
    fn OP_Fx25(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.memory[self.index as usize] = self.registers[Vx as usize];
    }
    ///Operator: rstr Vx
    /// grabs the memory adress at i and puts it in vx
    fn OP_Fx35(&mut self) {
        let Vx = (self.opcode & 0x0F00) >> 8;
        self.registers[Vx as usize] = self.registers[self.index as usize];
    }
    fn Cycle(&mut self) {
        //fetch memory and memory +1 cause op code is split into two bytes
        //Fetch first 8 bits, shift to the left 8 bits of the 16 bit opcode, then or the bottom bits to essentially combind them.
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;
        //memory location of next opcode
        self.pc += 2;
        let first_nibble = (self.opcode & 0xF000) >> 12 as u8;
        let second_nibble = (self.opcode & 0x0F00) >> 8;
        let third_nibble = (self.opcode & 0x00F0) >> 4 as u8;
        let last_nibble = (self.opcode & 0x000F) as u8;
        match (first_nibble) {
            0x0 => match (last_nibble) {
                0x0 => self.OP_00E0(),
                0x1 => self.OP_0151(),
                0x2 => self.OP_00F2(),
                0x8 => self.OP_0188(),
                0xE => self.OP_00EE(),
                0xD => self.OP_00ED(),
                _ => self.OP_NULL(),
            },

            0x1 => self.OP_1nnn(),
            0x2 => self.OP_2nnn(),
            0x3 => self.OP_3xkk(),
            0x4 => self.OP_4xkk(),
            0x5 => match (last_nibble) {
                0x0 => self.OP_5xy0(),
                0x1 => self.OP_5XY1(),
                0x2 => self.OP_5XY2(),
                0x3 => self.OP_5XY3(),
                _ => self.OP_NULL(),
            },
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
            //in theory, you could want to call Bnnn with the addr 0xB00 and itd call BBNN but i think the strat to that is not having programs over 1500 instructions long
            0xB => match (second_nibble) {
                0xB => self.OP_BBNN(),
                0xF => self.OP_BFNN(),
                _ => self.OP_Bnnn(),
            },

            0xC => self.OP_CxKK(),
            0xD => self.OP_Dxyn(),

            0xE => match (last_nibble) {
                0x1 => self.OP_ExA1(),
                0xE => self.OP_Ex9E(),
                _ => self.OP_NULL(),
            },

            0xF => match (last_nibble) {
                0xA => self.OP_Fx0A(),
                0xB => self.OP_FX1B(),
                0xF => self.OP_FX4F(),
                0x7 => match (third_nibble) {
                    0x0 => self.OP_Fx07(),
                    0xE => self.OP_FXE7(),
                    _ => self.OP_NULL(),
                },
                0x8 => self.OP_Fx18(),
                0xE => self.OP_Fx1E(),
                0x9 => self.OP_Fx29(),
                0x3 => match (third_nibble) {
                    0x0 => self.OP_FX03(),
                    0xE => self.OP_FXE3(),
                    0x3 => self.OP_Fx33(),
                    _ => self.OP_NULL(),
                },
                0x5 => match (third_nibble) {
                    0x1 => self.OP_Fx15(),
                    0x2 => self.OP_Fx25(),
                    0x3 => self.OP_Fx35(),
                    0x5 => self.OP_Fx55(),
                    0x6 => self.OP_Fx65(),
                    _ => self.OP_NULL(),
                },
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
    pub fn execute(&mut self, data: Vec<u8>) {
        self.load_array(data);
        loop {
            self.Cycle();
            if self.opcode == 0x0 {
                break;
            }
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
