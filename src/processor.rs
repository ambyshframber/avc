use std::io::{Write, stdout};
use std::fs::read;

use crate::utils::{bytes_to_16, u16_to_bytes, Options};

pub struct Processor {
    pub memory: [u8; 65536],
    pub a: u8,
    pub b: u8,
    pub x: u8,
    pub program_counter: usize, // so rust stops complaining
    pub status: u8,
    pub halted: bool,
    pub stack_pointer: usize, // see note on pc
    pub write_buffer: Box<dyn Write>
}

/// status flags:
/// 0: carry
/// 1: zero
/// 2: negative
/// 3: 

impl Default for Processor {
    fn default() -> Processor {
        Processor {
            memory: [0;65536],
            a: 0,
            b: 0,
            x: 0,
            program_counter: 0,
            status: 0,
            halted: false,
            stack_pointer: 0,
            write_buffer: Box::new(stdout()),
        }
    }
}

impl Processor {
    pub fn new_with_memory(mem: &[u8]) -> Processor {
        let mut p = Self::default();
        for (i,v) in mem.iter().enumerate() {
            p.memory[i] = *v
        }
        p
    }
    pub fn new(po: &Options) -> Processor {
        let mem = read(&po.path).unwrap();
        Self::new_with_memory(&mem)
    }
    #[allow(dead_code)]
    pub fn readout(&self) -> String {
        let mut ret = String::new();
        ret.push_str(&format!("a :   0x{:0>2x}\n", self.a));
        ret.push_str(&format!("b :   0x{:0>2x}\n", self.b));
        ret.push_str(&format!("x :   0x{:0>2x}\n", self.x));
        ret.push_str(&format!("pc: 0x{:0>4x}\n", self.program_counter));
        ret.push_str(&format!("sp: 0x{:0>4x}\n", self.stack_pointer));
        ret.push_str(&format!("      -----nzc\n"));
        ret.push_str(&format!("s : 0b{:0>8b}\n", self.status));

        ret
    }

    pub fn execute_until_halt(&mut self) {
        while !self.halted {
            self.execute();
        }
        println!("")
    }
    #[allow(dead_code)]
    pub fn execute_once(&mut self) -> bool {
        self.execute()
    }
    #[allow(dead_code)]
    pub fn execute_until_break(&mut self) {
        while !self.halted {
            if self.execute() {
                break
            }
        }
        println!("")
    }

    fn execute(&mut self) -> bool { // returns true if instr is break
        if self.program_counter > u16::MAX as usize {
            self.program_counter %= u16::MAX as usize
        }
        let instr = self.memory[self.program_counter];
        if false {
            println!("{}", instr);
            println!("{}", self.readout());
        }
        if instr == 23 {
            self.program_counter += 1;
            return true
        }

        match instr & 0b1000_0000 { // msb determines instruction width
            0 => { // leading zero = single width
                self.program_counter += 1;
                self.execute_single_width(instr);
            }
            _ => { // leading 1 = wide (in op op)
                self.execute_wide(instr);
                //self.program_counter += 3
            }
        }
        if self.a == 0 {
            self.status |= 0b10
        }
        else {
            self.status &= !0b10
        }

        false
    }
    fn execute_single_width(&mut self, instr: u8) {
        match instr & 0b0111_1111 { 
            0 => {} // nop
            1 => self.halted = true, //hlt
            //2 => print!("{}", self.a as char), // out
            2 => { // swp
                let tmp = self.a;
                self.a = self.b;
                self.b = tmp
            }
            3 => { // tab
                self.b = self.a
            }
            4 => { // tax
                self.x = self.a
            }
            5 => { // txa
                self.a = self.x
            }
            6 => { // inc
                self.x += 1
            }
            7 => { // dec
                self.x -= 1
            }
            8 => { // add
                let mut result = self.a as u16 + self.b as u16;
                if result > 255 {
                    self.status |= 0b1;
                    result &= 255
                }
                self.a = result as u8
            }
            9 => { // adc
                let result = self.a as u16 + self.b as u16 + (self.status & 1) as u16;
                self.status &= !1;
                if result & 255 != 0 {
                    self.status |= 0b1
                }
                self.a = result as u8
            }
            10 => { // sub
                if self.a >= self.b {
                    self.a = self.a - self.b
                }
                else {
                    self.a = self.b - self.a;
                    self.status |= 0b100
                }
            }
            11 => { // sbc
                let a = self.a as u16;
                let b = self.b as u16;
                if a >= b {
                    self.a = (a - b) as u8;
                    self.a += self.status & 1;
                    self.status &= !1;
                }
                else {
                    self.a = (b - a) as u8;
                    self.a += self.status & 1;
                    self.status |= 0b100
                }
            }
            12 => { // lsr
                if self.a & 128 != 0 {
                    self.status |= 0b1
                }
                self.a <<= 1
            }
            13 => { // lsl
                if self.a & 1 != 0 {
                    self.status |= 1
                }
                self.a >>= 1
            }
            14 => { // clc
                self.status &= !1
            }
            15 => { // sec
                self.status |= 1
            }
            16 => { // put
                let _ = self.write_buffer.write(&[self.a]);
            }
            17 => { // psa
                self.push(self.a)
            }
            18 => { // ppa
                self.a = self.pop()
            }
            19 => { // pss
                self.push(self.status)
            }
            20 => { // pps
                self.status = self.pop()
            }
            21 => { // ssp
                self.stack_pointer = bytes_to_16(self.a, self.b) as usize
            }
            22 => { // gsp
                (self.a, self.b) = u16_to_bytes(self.stack_pointer as u16)
            }
            23 => { // brk
                unreachable!() // debugging breakpoint
            }
            24 => { // rts
                let hi = self.pop();
                let lo = self.pop();
                self.program_counter = bytes_to_16(hi, lo) as usize
            }
            25 => { // lda const
                self.a = self.memory[self.program_counter];
                self.program_counter += 1
            }
            26 => { // get
                
            }
            27 => { // not
                self.a = !self.a
            }
            28 => { // and
                self.a &= self.b
            }
            29 => { // ior
                self.a |= self.b
            }
            30 => { // xor
                self.a ^= self.b
            }
            _ => {} // nop
        }
    }
    fn execute_wide(&mut self, instr: u8) {
        let op1 = self.memory[self.program_counter + 1];
        let op2 = self.memory[self.program_counter + 2];
        //dbg!(addr);
        self.program_counter += 3;
        //dbg!(self.program_counter);

        // for wide instructions:
        // lda 1xxx_x000
        // sta 1xxx_x001
        // jmp 1xxx_x010
        // jsr 1xxx_x011
        // jez 1xxx_x100
        // jgz 1xxx_x101

        // addressing modes:
        // direct 1xx0_0xxx
        // direct+offset 1xx0_1xxx
        // indirect 1xx1_0xxx
        // indirect+offset 1xx1_1xxx (gets addr, jumps to addr at addr+x)

        let addr = match instr & 0b0001_1000 {
            0b0000_0000 => bytes_to_16(op1, op2),
            0b0000_1000 => bytes_to_16(op1, op2) + self.x as u16,
            _ => {
                let tmp_addr = bytes_to_16(op1, op2);
                let hb = self.memory[tmp_addr as usize];
                let lb = self.memory[(tmp_addr + 1) as usize];
                let mut addr = bytes_to_16(hb, lb);
                if instr &0b1000 != 0 {
                    addr += self.x as u16
                }
                addr
            }
        } as usize;

        match instr & 0b0000_0111 { // add 128 to number
            0b000 => self.a = self.memory[addr],
            0b001 => self.memory[addr] = self.a,
            0b010 => self.program_counter = addr,
            0b011 => {
                let (hb, lb) = u16_to_bytes(self.program_counter as u16);
                self.push(lb);
                self.push(hb);
                self.program_counter = addr
            }
            0b100 => {
                if self.status & 0b10 != 0 {
                    self.program_counter = addr
                }
            }
            0b101 => {
                if self.a > 0 {
                    self.program_counter = addr
                }
            }
            _ => {} // nop
        }
    }

    fn push(&mut self, byte: u8) {
        //dbg!("pushing");
        self.memory[self.stack_pointer] = byte;
        self.stack_pointer += 1
    }
    fn pop(&mut self) -> u8 {
        //dbg!("popping");
        self.stack_pointer -= 1;
        self.memory[self.stack_pointer]
    }
}