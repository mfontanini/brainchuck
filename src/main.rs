use brainchuck::{parse, Codegen};
use inkwell::context::Context;

fn hello_world() -> &'static str {
    "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++."
}

fn main() {
    let context = Context::create();
    let codegen = Codegen::from_context(&context);
    let program = parse(hello_world()).expect("Invalid program");
    let result = codegen.run_program(&program, 50).expect("nope");
    println!(
        "Pointer ended at index {} and cell pointed at has value {}",
        result.pointer, result.value
    );
}
