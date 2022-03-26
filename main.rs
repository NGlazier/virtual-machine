#![warn(clippy::all)]

use std::env;
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;

use grumpy::{*, isa::*, vm::*};

fn main() -> io::Result<()> {
    // Read input file (command line argument at index 1).
    let path_str = env::args().nth(1).expect("missing file argument");
    let path = Path::new(&path_str);
    let mut file = OpenOptions::new().read(true).open(path)?;

    // Deserialize program from bytecode.
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let instrs = Vec::<Instr>::from_bytes(&mut bytes.into_iter())?;

    // Run program in VM.
    // match run(Debug::DEBUG, &instrs) {
    match run(Debug::DEBUG, &instrs) {
        Ok(v) => print!("{:?}", v),
        Err(msg) => {
            print!("{}", msg);
            exit(1)
        }
    }
    
    Ok(())
}
