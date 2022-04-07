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
            assembler::assemble(&po)
        }
        Command::Run => {
            let mut p = Processor::new(&po);
            //println!("{}", p.memory[1]);
            p.execute_until_halt()
        }
    }

    Ok(())
}

fn get_options() -> Options {
    let mut o = Options::default();

    {
        let mut ap = ArgumentParser::new();
        
        ap.refer(&mut o.command)
            .add_option(&["-A"], StoreConst(Command::Assemble), "assemble")
            .add_option(&["-R"], StoreConst(Command::Run), "run")
        ;
        ap.refer(&mut o.path).add_argument("file", Store, "the file to run/assemble");

        ap.parse_args_or_exit()
    }

    o
}
