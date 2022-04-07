use std::collections::HashMap;

use crate::utils::{u16_to_bytes, parse_int_literal, set_vec_value_at_index};

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

        let main_instr = main_instr.split(';').next().unwrap(); // ignore comments
        let split_index = main_instr.find(' ');
        let (instr, op) = match split_index {
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
            
            "lda"|"sta"|"org"|"dat"|"blk"|"jmp"|"jsr"|"jez" => { // wide
                if op == "" { // check operand exists
                    return Err(format!("instr {} requires op, found none", instr))
                }
                let is_literal = op.starts_with('#'); // check if literal or offset
                let is_offset = op.ends_with(",x");
                let is_label = op.starts_with('%');
                if is_literal && is_offset {
                    return Err(format!("operand cannot be literal and offset!"))
                }
                let is_string = op.starts_with('"');
                let mut lit = if is_literal || is_label { // trim markers
                    &op[1..]
                }
                else if is_string {
                    if instr != "dat" {
                        return Err(format!("string operand on non-dat instruction!"))
                    }
                    if !op.ends_with('"') {
                        return Err(format!("string operand without closing quote!"))
                    }
                    &op[1..op.len() - 1]
                }
                else {
                    op
                };
                if is_offset {
                    lit = &lit[..lit.len() - 2]
                }
                let operand = if is_label || is_string {
                    0
                }
                else {
                    match parse_int_literal(lit) {
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    }
                };
                self.counter += 1; // bump counter
                match instr {
                    "lda" => {
                        if is_literal {
                            line.instruction = I::LdaConst;
                            line.operand = Op::Byte(operand as u8);
                        }
                        else {
                            if is_offset {
                                line.instruction = I::LdaAddrOffset
                            }
                            else {
                                line.instruction = I::LdaAddr
                            }
                            if is_label {
                                line.operand = Op::Label(String::from(lit))
                            }
                            else {
                                line.operand = Op::Addr(operand);
                            }
                            self.counter += 1; // bump counter
                        }
                    }
                    "sta" => {
                        if is_offset {
                            line.instruction = I::StaAddrOffset
                        }
                        else {
                            line.instruction = I::StaAddr
                        }
                        if is_label {
                            line.operand = Op::Label(String::from(lit))
                        }
                        else {
                            line.operand = Op::Addr(operand);
                        }
                        self.counter += 1; // bump counter
                    }
                    "jmp" => {
                        if is_offset {
                            line.instruction = I::JmpAddrOffset
                        }
                        else {
                            line.instruction = I::JmpAddr
                        }
                        if is_label {
                            line.operand = Op::Label(String::from(lit))
                        }
                        else {
                            line.operand = Op::Addr(operand);
                        }
                        self.counter += 1; // bump counter
                    }
                    "jsr" => {
                        if is_offset {
                            line.instruction = I::JsrAddrOffset
                        }
                        else {
                            line.instruction = I::JsrAddr
                        }
                        if is_label {
                            line.operand = Op::Label(String::from(lit))
                        }
                        else {
                            line.operand = Op::Addr(operand);
                        }
                        self.counter += 1; // bump counter
                    }
                    "jez" => {
                        if is_offset {
                            line.instruction = I::JezAddrOffset
                        }
                        else {
                            line.instruction = I::JezAddr
                        }
                        if is_label {
                            line.operand = Op::Label(String::from(lit))
                        }
                        else {
                            line.operand = Op::Addr(operand);
                        }
                        self.counter += 1; // bump counter
                    }
                
                    "org" => {
                        line.instruction = I::Org;
                        line.operand = Op::Addr(operand);
                        self.counter = operand as usize;
                    }
                    "dat" => {
                        line.instruction = I::Dat;
                        if is_string {
                            let bytes = lit.bytes().collect::<Vec<u8>>();
                            self.counter += bytes.len() - 1; // i don't know why the -1 is needed, but it is.
                            line.operand = Op::ByteBlock(bytes.into_boxed_slice())
                        }
                        else {
                            line.operand = Op::Byte(operand as u8);
                        }
                        self.counter -= 1
                    }
                    _ => {
                        unreachable!()
                    }
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
                    match self.labels.get(l) {
                        Some(a) => {
                            //dbg!(*a as u16);
                            let (hi, lo) = u16_to_bytes(*a as u16);
                            ret.push(hi);
                            ret.push(lo)
                        }
                        None => {
                            return Err(format!("unrecognised label {}", l))
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
#[derive(PartialEq, Clone)]
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
    Brk, Rts, LdaConst,

    // wide ops
    LdaAddr = 128, StaAddr,
    LdaAddrOffset, StaAddrOffset,
    JmpAddr, JmpAddrOffset,
    JsrAddr, JsrAddrOffset,
    JezAddr, JezAddrOffset,

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