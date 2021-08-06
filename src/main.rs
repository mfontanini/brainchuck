use brainchuck::{parse, Codegen};
use inkwell::context::Context;

fn hello_world() -> &'static str {
    "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++."
}

fn main() {
    let context = Context::create();
    let codegen = Codegen::from_context(&context);
    let program = parse(hello_world()).expect("Invalid program");
    if let Err(e) = codegen.run_program(&program, 50) {
        eprintln!("Program execution failed: {}", e);
    }
}
