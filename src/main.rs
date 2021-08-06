use brainchuck::{parse, Codegen, Command};
use inkwell::context::Context;
use std::{
    error::Error,
    fs::File,
    io::{self, Read},
    process::exit,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    /// Print IR code instead of running program
    #[structopt(short, long)]
    print_code: bool,

    /// The source file to read the program from. Use "-" to read it from stdin
    #[structopt(name = "source")]
    source: String,
}

fn build_source(source: &str) -> Result<Box<dyn Read>, Box<dyn Error>> {
    match source {
        "-" => Ok(Box::new(io::stdin())),
        filename => {
            let file = File::open(filename)?;
            Ok(Box::new(file))
        }
    }
}

fn parse_program<R: Read>(mut input: R) -> Result<Vec<Command>, Box<dyn Error>> {
    let mut contents = String::new();
    input.read_to_string(&mut contents)?;
    let program = parse(&contents)?;
    Ok(program)
}

fn run(codegen: Codegen, program: &[Command], memory_size: u16) {
    if let Err(e) = codegen.run_program(&program, memory_size) {
        eprintln!("Program execution failed: {}", e);
    }
}

fn print_code(codegen: Codegen, program: &[Command], memory_size: u16) {
    match codegen.generate_code(&program, memory_size) {
        Ok(code) => println!("{}", code),
        Err(e) => eprintln!("Program execution failed: {}", e),
    };
}

fn main() {
    let options = Options::from_args();
    let source = match build_source(&options.source) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error reading program: {}", e);
            exit(1);
        }
    };
    let program = match parse_program(source) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error parsing program: {}", e);
            exit(1);
        }
    };

    let context = Context::create();
    let codegen = Codegen::from_context(&context);
    if options.print_code {
        print_code(codegen, &program, 50);
    } else {
        run(codegen, &program, 50);
    }
}
