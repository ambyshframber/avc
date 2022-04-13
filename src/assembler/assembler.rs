use std::collections::HashMap;
use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;

use crate::utils::{u16_to_bytes, parse_int_literal, set_vec_value_at_index, strip_whitespace};

// parse program into Lines
// 

#[derive(Default)]
struct Assembler {
    lines: Vec<Line>,
    pub labels: HashMap<String, usize>,
    counter: usize
}

pub fn assemble(program: &str) -> Result<Box<[u8]>, String> {
    let mut a = Assembler::default();

    for (i, l) in program.split('\n').enumerate() {
        match a.read_line(l, i) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("error on line {}: {}", i + 1, e))
            }
        }
    }
    
    match a.compile() {
        Ok(bytes) => {
            for label in a.labels.keys() {
                println!("label {}, pointing to {} at {}", label, bytes[a.labels[label]], a.labels[label])
            }
            Ok(bytes)
        }
        Err(e) => Err(format!("error compiling: {}", e))
    }
}
impl Assembler {
    pub fn read_line(&mut self, s: &str, index: usize) -> Result<(), String> { // index is line number. this is the worst way of doing it, i know
        if s.trim() == "" {
            return Ok(())
        }
        let s = s.split(';').next().unwrap(); // ignore comments
        let parts_initial = s.trim_start().split(':'); // split off label
        let parts = parts_initial.collect::<Vec<&str>>();
        let main_instr = if parts.len() == 2 { // if there is a label, add it
            self.labels.insert(String::from(parts[0].trim()), self.counter);
            let main = parts[1].trim();
            if main == "" {
                return Ok(())
            }
            main
        }
        else {
            parts[0]
        };
        
        self.counter += 1; // bump counter

        //let main_instr = main_instr.split(';').next().unwrap(); // ignore comments
        let split_index = main_instr.find(' '); // find space
        let (instr, op) = match split_index { // split at space
            Some(i) => {
                main_instr.split_at(i)
            }
            None => {
                (main_instr, "")
            }
        };
        let op = op.trim();

        if parts.len() == 0 {
            panic!()
        }
        
        type I = Instruction; // make the code easier to write
        type Op = Operand;
        let mut line = Line {
            instruction: I::Nop,
            operand: Op::None,
            program_text_line: index
        };
        match instr {
            "nop" => line.instruction = I::Nop,
            "hlt" => line.instruction = I::Hlt,
            "swp" => line.instruction = I::Swp,
            "tab" => line.instruction = I::Tab,
            "tax" => line.instruction = I::Tax,
            "txa" => line.instruction = I::Txa,
            "inc" => line.instruction = I::Inc,
            "dec" => line.instruction = I::Dec,
            "add" => line.instruction = I::Add,
            "adc" => line.instruction = I::Adc,
            "sub" => line.instruction = I::Sub,
            "sbc" => line.instruction = I::Sbc,
            "lsr" => line.instruction = I::Lsr,
            "lsl" => line.instruction = I::Lsl,
            "clc" => line.instruction = I::Clc,
            "sec" => line.instruction = I::Sec,
            "put" => line.instruction = I::Put,
            "psa" => line.instruction = I::Psa,
            "ppa" => line.instruction = I::Ppa,
            "pss" => line.instruction = I::Pss,
            "pps" => line.instruction = I::Pps,
            "ssp" => line.instruction = I::Ssp,
            "gsp" => line.instruction = I::Gsp,
            "brk" => line.instruction = I::Brk,
            "rts" => line.instruction = I::Rts,
            
            "lda"|"sta"|"org"|"dat"|"jmp"|"jsr"|"jez"|"jgz" => { // wide
                if op == "" { // check operand exists
                    return Err(format!("instr {} requires op, found none", instr))
                }
                //dbg!(op);
                // dat "string"
                // lda (addr,x)
                // lda #bb
                let is_literal = op.starts_with('#');
                let is_string = op.starts_with('"');
                let is_indirect = op.starts_with('(');
                let mut literal = op;
                if is_string {
                    if !op.ends_with('"') {
                        return Err(String::from("string operand with no close quote!"))
                    }
                    else {
                        literal = &literal[1..literal.len() - 1]
                    }
                }
                if is_indirect {
                    if !op.ends_with(')') {
                        return Err(String::from("indirect address operand with no close bracket!"))
                    }
                    else {
                        literal = &literal[1..literal.len() - 1]
                    }
                }
                let is_offset = literal.ends_with(",x");
                if is_literal {
                    if instr != "lda" {
                        return Err(String::from("literal operand not valid for non-lda instructions!"))
                    }
                    if is_offset {
                        return Err(String::from("literal operand cannot be offset!"))
                    }
                    literal = &literal[1..];
                }
                if is_offset {
                    literal = &literal[..literal.len() - 2]
                }
                //dbg!(literal);
                let (operand, is_label) = match parse_int_literal(literal) {
                    Ok(v) => (v, false),
                    Err(_) => (0, true)
                };
                let mut instr_output = 0b1000_0000;
                if is_literal {
                    if is_label {
                        return Err(String::from("literal operand failed to parse!"))
                    }
                    line.instruction = I::LdaConst;
                    line.operand = Op::Byte(operand as u8);
                    self.counter += 1
                }
                else {
                    match instr {
                        "org" => {
                            instr_output = I::Org as u8;
                            line.operand = Op::Addr(operand);
                            self.counter = operand as usize;
                        }
                        "dat" => {
                            instr_output = I::Dat as u8;
                            if is_string {
                                let bytes = literal.bytes().collect::<Vec<u8>>();
                                self.counter += bytes.len() - 1; // i don't know why the -1 is needed, but it is.
                                line.operand = Op::ByteBlock(bytes.into_boxed_slice())
                            }
                            else {
                                line.operand = Op::Byte(operand as u8);
                            }
                        }
                        _ => {
                            self.counter += 2;
                            instr_output |= match instr {
                                "lda" => 0b000,
                                "sta" => 0b001,
                                "jmp" => 0b010,
                                "jsr" => 0b011,
                                "jez" => 0b100,
                                "jgz" => 0b101,
                                _ => unreachable!()
                            };
                            if is_label {
                                line.operand = Op::Label(String::from(literal))
                            }
                            else {
                                line.operand = Op::Addr(operand)
                            }
                            if is_offset {
                                instr_output |= 0b1000
                            }
                            if is_indirect {
                                instr_output |= 0b1_0000
                            }
                        }
                    }
                    line.instruction = FromPrimitive::from_u8(instr_output).unwrap()
                }

            }
            _ => {
                return Err(format!("unrecognised instruction {}!", instr))
            }
        }

        self.lines.push(line);

        Ok(())
    }
    pub fn compile(&mut self) -> Result<Box<[u8]>, String> {
        let mut ret = Vec::new();

        self.counter = 0;
        for l in &self.lines {
            if l.instruction == Instruction::Org {
                self.counter = match l.operand {
                    Operand::Addr(a) => a as usize,
                    _ => unreachable!()
                };
                continue
            }
            let instr = match self.line_to_bytes(l) {
                Ok(bytes) => {
                    bytes.into_vec()
                }
                Err(e) => {
                    return Err(format!("line {}: {}", l.program_text_line, e))
                }
            };
            for b in instr {
                set_vec_value_at_index(&mut ret, b, self.counter);
                self.counter += 1
            }
        }
        
        Ok(ret.into_boxed_slice())
    }
    fn line_to_bytes(&self, line: &Line) -> Result<Box<[u8]>, String> {
        let mut ret = Vec::new();

        type Op = Operand;
        if line.instruction == Instruction::Dat {
            match &line.operand {
                Operand::Byte(b) => ret.push(*b),
                Operand::ByteBlock(b) => {
                    for byte in b.clone().into_vec() {
                        ret.push(byte)
                    }
                }
                _ => unreachable!()
            }
        }
        else {
            let instr = line.instruction.clone() as u8;
            ret.push(instr);
            match &line.operand {
                Op::Addr(a) => {
                    let (hi, lo) = u16_to_bytes(*a);
                    ret.push(hi);
                    ret.push(lo)
                }
                Op::Byte(b) => {
                    ret.push(*b)
                }
                Op::Label(l) => {
                    // ARITHMETIC HERE
                    let label_strip = strip_whitespace(l);
                    let mut label = label_strip.as_str();
                    let addition = if l.contains(['+', '-']) { // split on either
                        let is_subtraction = l.contains('-'); // check if it's neg
                        let mut split = label.split(['+', '-']);
                        label = split.next().unwrap(); // string is not empty (i checked)
                        match split.next() {
                            Some(v) => {
                                match parse_int_literal::<i32>(v) {
                                    Ok(v) => v * if is_subtraction {-1} else {1},
                                    Err(e) => return Err(e)
                                }
                            }
                            None => return Err(format!("no value found after const additon: {}", l))
                        }
                    }
                    else {0};

                    match self.labels.get(label) {
                        Some(a) => {
                            //dbg!(*a as u16);
                            let (hi, lo) = u16_to_bytes(((*a as i32) + addition) as u16);
                            ret.push(hi);
                            ret.push(lo)
                        }
                        None => {
                            return Err(format!("unrecognised label {}", label))
                        }
                    }
                }
                Op::None => {}
                _ => unreachable!()
            }
        }

        Ok(ret.into_boxed_slice())
    }
}

struct Line {
    pub instruction: Instruction,
    pub operand: Operand,
    pub program_text_line: usize
}
enum Operand {
    None,
    Byte(u8),
    Addr(u16),
    Label(String),
    #[allow(dead_code)]
    ByteBlock(Box<[u8]>)
}
impl Default for Line {
    fn default() -> Line {
        Line {
            instruction: Instruction::Nop,
            operand: Operand::None,
            program_text_line: 0
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Clone, FromPrimitive)]
enum Instruction {
    // processor ops
    // control
    Nop, Hlt,

    // internal register mgmt
    Swp, Tab, Tax, Txa, Inc, Dec,

    // arithmetic
    Add, Adc, Sub, Sbc, Lsr, Lsl,

    // carry
    Clc, Sec,

    // output
    Put,

    // stack
    Psa, Ppa,
    Pss, Pps, Ssp, Gsp,

    // misc
    Brk, Rts, LdaConst, Get,

    // bitwise
    Not, And, Ior, Xor,

    // wide ops
    LdaAddr = 0b1000_0000,
    StaAddr, JmpAddr, JsrAddr, JezAddr, JgzAddr,
    LdaAddrOffset = 0b1000_1000,
    StaAddrOffset, JmpAddrOffset, JsrAddrOffset, JezAddrOffset, JgzAddrOffset,
    LdaInd = 0b1001_0000,
    StaInd, JmpInd, JsrInd, JezInd, JgzInd, 
    LdaIndOffset = 0b1001_1000,
    StaIndOffset, JmpIndOffset, JsrIndOffset, JezIndOffset, JgzIndOffset, 

    // assembler directives
    Org, Dat
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(parse_int_literal("0b10"), Ok(2));
        assert_eq!(parse_int_literal("0x10"), Ok(16));
        assert_eq!(parse_int_literal("0d10"), Ok(10));
    }
}