use std::fs::{read_to_string, write};

use crate::utils::Options;

mod assembler;

pub fn assemble(po: &Options) {
    let program = read_to_string(&po.path).unwrap();
    let assembly = assembler::assemble(&program);
    let _ = write("a.out", assembly.unwrap());
}