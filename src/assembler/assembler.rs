use std::collections::HashMap;

use crate::utils::{u16_to_bytes, parse_int_literal};

// parse program into Lines
// 

#[derive(Default)]
struct Assembler {
    lines: Vec<Line>,
    labels: HashMap<String, usize>,
    counter: usize
}

pub fn assemble(program: &str) -> Result<Box<[u8]>, String> {
    let mut a = Assembler::default();

    for (i, l) in program.split('\n').enumerate() {
        match a.read_line(l) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("error on line {}: {}", i + 1, e))
            }
        }
    }
    match a.compile() {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(format!("error compiling: {}", e))
    }
}
impl Assembler {
    pub fn read_line(&mut self, s: &str) -> Result<(), String> {
        if s.trim() == "" {
            self.lines.push(Line::default());
            return Ok(())
        }
        let parts_initial = s.trim_start().split(':'); // split off label
        let parts = parts_initial.collect::<Vec<&str>>();
        let main_instr = if parts.len() == 2 { // if there is a label, add it
            self.labels.insert(String::from(parts[0].trim()), self.counter);
            parts[1].trim_start()
        }
        else {
            parts[0]
        };
        self.counter += 1; // bump counter

        let parts = main_instr.split(' ').collect::<Vec<&str>>(); // split main body of instr

        if parts.len() == 0 {
            panic!()
        }
        
        type I = Instruction; // make the code easier to write
        type Op = Operand;
        let mut line = Line {
            instruction: I::Nop,
            operand: Op::None
        };
        match parts[0] {
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
            "out" => line.instruction = I::Out,
            "psa" => line.instruction = I::Psa,
            "ppa" => line.instruction = I::Ppa,
            "pss" => line.instruction = I::Pss,
            "pps" => line.instruction = I::Pps,
            "ssp" => line.instruction = I::Ssp,
            "gsp" => line.instruction = I::Gsp,
            "brk" => line.instruction = I::Brk,
            "rts" => line.instruction = I::Rts,
            
            "lda"|"sta"|"org"|"dat"|"blk"|"jmp"|"jsr" => { // wide
                if parts.len() == 1 { // check operand exists
                    return Err(format!("instr {} requires op, found none", parts[0]))
                }
                let is_literal = parts[1].starts_with('#'); // check if literal or offset
                let is_offset = parts[1].ends_with(",x");
                let is_label = parts[1].starts_with('%');
                if is_literal && is_offset {
                    return Err(format!("operand cannot be literal and offset!"))
                }
                let lit = if is_literal || is_label { // trim markers
                    &parts[1][1..]
                }
                else {
                    parts[1]
                };
                let lit = if is_offset {
                    &lit[..lit.len() - 2]
                }
                else {
                    lit
                };
                let operand = if is_label {
                    0
                }
                else {
                    match parse_int_literal(lit) { // TODO
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    }
                };
                self.counter += 1; // bump counter
                match parts[0] {
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
                
                    "org" => {
                        line.instruction = I::Org;
                        line.operand = Op::Addr(operand);
                        self.counter = operand as usize;
                    }
                    "dat" => {
                        line.instruction = I::Dat;
                        line.operand = Op::Byte(operand as u8);
                        self.counter -= 1
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            _ => {
                return Err(format!("unrecognised instruction {}!", parts[0]))
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
            while self.counter > ret.len() {
                ret.push(0)
            }
            match self.line_to_bytes(l) {
                Ok(bytes) => {
                    ret.append(&mut bytes.into_vec())
                }
                Err(e) => {
                    return Err(e)
                }
            }
        }

        
        Ok(ret.into_boxed_slice())
    }
    fn line_to_bytes(&self, line: &Line) -> Result<Box<[u8]>, String> {
        let mut ret = Vec::new();

        type Op = Operand;
        if line.instruction == Instruction::Dat {
            ret.push(match line.operand {
                Operand::Byte(b) => b,
                _ => unreachable!()
            })
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
    pub operand: Operand
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
            operand: Operand::None
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Clone)]
enum Instruction {
    // processor ops
    // control
    Nop,
    Hlt,

    // internal register mgmt
    Swp,
    Tab,
    Tax,
    Txa,
    Inc,
    Dec,

    // arithmetic
    Add,
    Adc,
    Sub,
    Sbc,
    Lsr,
    Lsl,

    // carry
    Clc,
    Sec,

    // output
    Out,

    // stack
    Psa,
    Ppa,
    Pss,
    Pps,
    Ssp,
    Gsp,

    // misc
    Brk,
    Rts,
    LdaConst,

    // wide ops
    LdaAddr = 128,
    StaAddr,
    LdaAddrOffset,
    StaAddrOffset,
    JmpAddr,
    JmpAddrOffset,
    JsrAddr,
    JsrAddrOffset,

    // assembler directives
    Org,
    Dat
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