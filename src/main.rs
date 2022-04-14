use argparse::{ArgumentParser, Store, StoreConst};
use std::process::exit;

use processor::Processor;
use utils::{Options, Command};

mod processor;
mod utils;
mod assembler;

fn main() {
    match run_program() {
        Ok(_) => exit(0),
        Err((e, s)) => {
            println!("{}", s);
            exit(e)
        }
    }
}

fn run_program() -> Result<(), (i32, String)> {
    let po = get_options();

    match po.command {
        Command::Assemble => {
            match assembler::assemble(&po) {
                Ok(_) => {}
                Err(e) => return Err((1, e))
            }
        }
        Command::Run => {
            let mut p = Processor::new(&po);
            p.run(&po)
        }
    }

    Ok(())
}

fn get_options() -> Options {
    let mut o = Options::default();
    o.out_path = String::from("a.out");

    {
        let mut ap = ArgumentParser::new();
        
        ap.refer(&mut o.command)
            .add_option(&["-A"], StoreConst(Command::Assemble), "assemble")
            .add_option(&["-R"], StoreConst(Command::Run), "run")
        ;
        ap.refer(&mut o.out_path).add_option(&["-o"], Store, "Output file path (for assembly)");
        ap.refer(&mut o.debug_level).add_option(&["-d"], Store, "Debug level. 0 is none, 1 is readout on break, 2 is 1+instructions, 3 is readout every cycle");
        ap.refer(&mut o.path).add_argument("file", Store, "the file to run/assemble");

        ap.parse_args_or_exit()
    }
    if o.debug_level > 3 {
        println!("invalid debug level {}", o.debug_level);
        exit(2)
    }

    o
}
