use brainchuck::{parse, Codegen, Command};
use inkwell::context::Context;
use structopt::StructOpt;

fn hello_world() -> &'static str {
    "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++."
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    /// Print IR code instead of running program
    #[structopt(short, long)]
    print_code: bool,
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
    let context = Context::create();
    let codegen = Codegen::from_context(&context);
    let program = parse(hello_world()).expect("Invalid program");

    if options.print_code {
        print_code(codegen, &program, 50);
    } else {
        run(codegen, &program, 50);
    }
}
