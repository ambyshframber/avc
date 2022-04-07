use std::fs::{read_to_string, write};

use crate::utils::Options;

mod assembler;

pub fn assemble(po: &Options) -> Result<(), String> {
    let program = match read_to_string(&po.path) {
        Ok(s) => s,
        Err(_) => return Err(format!("unable to read file {}", po.path))
    };
    let assembly = match assembler::assemble(&program) {
        Ok(a) => a,
        Err(e) => return Err(e)
    };
    match write(&po.out_path, assembly) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("unable to write file {}", po.out_path))
    }
}