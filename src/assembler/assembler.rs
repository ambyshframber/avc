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
    counter: usize,
    pub constants: HashMap<String, u8>,
    macros: HashMap<String, Vec<String>>
}

pub fn assemble(program: &str) -> Result<Box<[u8]>, String> {
    let mut a = Assembler::default();

    let mut lines = program.split('\n').peekable();
    let mut i = 1;

    let mut declarations = Vec::new();
    if lines.peek().unwrap().starts_with('#') {
        loop { // PASS 1: define/macro parsing
            let l = lines.next().unwrap();
            if l == "#ENDD" {
                i += 1;
                break
            }
            if l.trim() != "" { // don't add empty lines
                declarations.push(String::from(l));
            }
            i += 1;
        }
    }
    /*if declarations.len() != 0 {
        println!("found declarations:");
        for d in &declarations {
            println!("\t{}", d)
        }
    }*/
    a.process_declares(declarations)?;

    loop { // PASS 2: line parsing
        let l = match lines.next() {
            Some(l) => l,
            None => break
        };
        match a.read_line(l, i) {
            Ok(opt) => {
                match opt {
                    Some(l) => a.lines.push(l),
                    None => {}
                }
            }
            Err(e) => {
                return Err(format!("error on line {}: {}", i + 1, e))
            }
        }
    }
    
    match a.compile() { // PASS 3: compiling
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
    pub fn process_declares(&mut self, decs: Vec<String>) -> Result<(), String> {
        let mut declarations = decs.iter();

        loop {
            let d = match declarations.next() {
                Some(v) => v, None => break
            };
            if d.starts_with(';') {
                continue
            }

            match &d[..5] { // #DCLR
                "#BYTE" => { // #BYTE name val
                    let mut s = d.split(' ');
                    let _ = s.next();
                    let name = match s.next() {
                        Some(v) => v, None => return Err(format!("declaration \"{}\" missing name", d))
                    };
                    let val_str = match s.next() {
                        Some(v) => v, None => return Err(format!("declaration \"{}\" missing value", d))
                    };
                    let val = parse_int_literal::<u8>(val_str)?;
                    self.constants.insert(String::from(name), val);
                }
                "#MACR" => { // can have arguments
                    let mut mac_lines = Vec::new();
                    loop { // collect all lines until "#ENDM" into a vec
                        match declarations.next() {
                            Some(v) => {
                                if !v.starts_with("#ENDM") {
                                    mac_lines.push(String::from(v))
                                }
                                else {
                                    break
                                }
                            }
                            None => {
                                return Err(format!("macro without close tag"))
                            }
                        }
                    }
                    let mac_name = &d[5..].trim();
                    let mac_name = mac_name.split([';', ' ']).next().unwrap().trim();
                    //println!("{}", mac_name);
                    self.macros.insert(String::from(mac_name), mac_lines);
                }
                _ => return Err(format!("unrecognised declaration \"{}\"", d))
            }
        }

        Ok(())
    }

    pub fn read_line(&mut self, s: &str, index: usize) -> Result<Option<Line>, String> { // index is line number. this is the worst way of doing it, i know
        let s = s.trim();
        if s == "" { // ignore empty strings
            return Ok(None)
        }
        if s.starts_with('#') {
            return Err(format!("directive \"{}\" after the start of instructions", s))
        }
        let s = s.split(';').next().unwrap(); // ignore comments
        if s.trim() == "" { // ignore just comments
            return Ok(None)
        }
        let parts_initial = s.trim_start().split(':'); // split off label
        let parts = parts_initial.collect::<Vec<&str>>();
        let main_instr = if parts.len() == 2 { // if there is a label, add it
            self.labels.insert(String::from(parts[0].trim()), self.counter);
            let main = parts[1].trim();
            if main == "" { // ignore just labels
                return Ok(None)
            }
            main
        }
        else {
            parts[0]
        };
        if s.starts_with('!') { // MACRO
            self.expand_macro(main_instr, index)?;
            return Ok(None)
        }
        
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
            program_text_line: index + 1
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
            //"adc" => line.instruction = I::Adc,
            //"sub" => line.instruction = I::Sub,
            //"sbc" => line.instruction = I::Sbc,
            "lsr" => line.instruction = I::Lsr,
            "lsl" => line.instruction = I::Lsl,
            "clc" => line.instruction = I::Clc,
            "sec" => line.instruction = I::Sec,
            "put" => line.instruction = I::Put,
            "psa" => line.instruction = I::Psa,
            "ppa" => line.instruction = I::Ppa,
            //"sst" => line.instruction = I::Sst,
            //"gst" => line.instruction = I::Sst,
            "ssp" => line.instruction = I::Ssp,
            "gsp" => line.instruction = I::Gsp,
            "brk" => line.instruction = I::Brk,
            "rts" => line.instruction = I::Rts,
            "get" => line.instruction = I::Get,
            "gbf" => line.instruction = I::Gbf,
            "not" => line.instruction = I::Not,
            "and" => line.instruction = I::And,
            "ior" => line.instruction = I::Ior,
            "xor" => line.instruction = I::Xor,
            
            "lda"|"sta"|"org"|"dat"|"jmp"|"jsr"|"jez"|"jgt" => { // jgz is gone :crab: :crab:
                if op == "" { // check operand exists
                    return Err(format!("instr {} requires op, found none", instr))
                }
                //dbg!(op);
                // dat "string"
                // lda (addr,x)
                // lda #bb
                let (literal, operand, is_label, is_offset, is_indirect, is_string, is_literal) = parse_op(op)?;
                
                if is_literal && instr != "lda" {
                    return Err(format!("literal operands are not allowed for non-lda instructions"))
                }
                if is_literal { // lda #bb
                    line.instruction = I::LdaConst;
                    line.operand = Op::Byte(if is_label {
                        match self.constants.get(literal) {
                            Some(v) => *v, None => return Err(format!("undefined constant {}", literal))
                        }
                    }
                    else {
                        operand as u8
                    });
                    self.counter += 1
                }
                else { // xyz hhll
                    let mut instr_output = 0b1000_0000;
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
                                line.operand = Op::ByteBlock(bytes)
                            }
                            else if is_label { // didn't parse as number
                                let s = literal.split(',');
                                let mut bytes = Vec::new();
                                for num in s {
                                    bytes.push(self.get_val_from_string(num)?)
                                }
                                self.counter += bytes.len() - 1;
                                line.operand = Op::ByteBlock(bytes)
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
                                "jgt" => 0b101,
                                _ => unreachable!()
                            };
                            if is_label {
                                if !literal.is_ascii() {
                                    return Err(format!("string literal \"{}\" contains non-ascii characters", literal))
                                }
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
                return Err(format!("unrecognised instruction {}", instr))
            }
        }

        //self.lines.push(line);

        Ok(Some(line))
    }

    #[allow(unused_variables)]
    fn expand_macro(&mut self, line: &str, index: usize) -> Result<(), String> {
        // comments/labels already stripped out
        let mut args = line.trim().split(' ');
        
        let mac = &args.next().unwrap()[1..];
        let mac_lines = match self.macros.get(mac) { // remove '!'
            Some(v) => v,
            None => return Err(format!("macro {} not found", mac))
        }.clone();
        let mut replacements = HashMap::new();
        for (i, a) in args.enumerate() {
            let rep = format!("${}", i + 1);
            replacements.insert(rep, a);
        }
        for mut l in mac_lines {
            for (key, val) in &replacements {
                l = l.replace(key, val)
            }
            match self.read_line(&l, 0) {
                Ok(v) => {
                    match v {
                        Some(v) => self.lines.push(v), None => {}
                    }
                }
                Err(e) => return Err(format!("error in macro {}: {}", mac, e))
            }
        }


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
                    for byte in b {
                        ret.push(*byte)
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

    fn get_val_from_string(&self, s: &str) -> Result<u8, String> { // returns u8 in lb and false if it's u8
        let s = s.trim();
        match parse_int_literal::<u8>(s) {
            Ok(v) => Ok(v),
            Err(_) => {
                match self.constants.get(s) {
                    Some(v) => Ok(*v),
                    None => Err(format!("undefined constant {}", s))
                }
            }
        }
    }
}

fn parse_op(op: &str) -> Result<(&str, u16, bool, bool, bool, bool, bool), String> {
    let mut literal = op;
    let is_literal = literal.starts_with('#');
    let is_offset = literal.ends_with(",x");
    if is_literal {
        if is_offset {
            return Err(String::from("literal operand cannot be offset"))
        }
        literal = &literal[1..];
    }
    if is_offset {
        literal = &literal[..literal.len() - 2]
    }
    let is_string = literal.starts_with('"');
    let is_indirect = literal.starts_with('(');
    if is_string {
        if !literal.ends_with('"') {
            return Err(String::from("string operand with no close quote"))
        }
        else {
            literal = &literal[1..literal.len() - 1]
        }
    }
    if is_indirect {
        if !literal.ends_with(')') {
            return Err(String::from("indirect address operand with no close bracket"))
        }
        else {
            literal = &literal[1..literal.len() - 1]
        }
    }
    //dbg!(literal);
    let (operand, is_label) = match parse_int_literal(literal) {
        Ok(v) => (v, false),
        Err(_) => (0, true)
    };
    Ok((literal, operand, is_label, is_offset, is_indirect, is_string, is_literal))
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
    ByteBlock(Vec<u8>)
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
    Sst, Gst, // DEPRECATED
    Ssp, Gsp,

    // misc
    Brk, Rts, LdaConst, Get,

    // bitwise
    Not, And, Ior, Xor,

    Gbf,

    // wide ops
    LdaAddr = 0b1000_0000,
    StaAddr, JmpAddr, JsrAddr, JezAddr, JgtAddr,
    LdaAddrOffset = 0b1000_1000,
    StaAddrOffset, JmpAddrOffset, JsrAddrOffset, JezAddrOffset, JgtAddrOffset,
    LdaInd = 0b1001_0000,
    StaInd, JmpInd, JsrInd, JezInd, JgtInd, 
    LdaIndOffset = 0b1001_1000,
    StaIndOffset, JmpIndOffset, JsrIndOffset, JezIndOffset, JgtIndOffset, 

    // assembler directives
    Org, Dat
}
