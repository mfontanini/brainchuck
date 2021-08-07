use crate::parser::Command;
use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{FunctionLookupError, JitFunction},
    module::{Linkage, Module},
    support::LLVMString,
    values::{FunctionValue, IntValue, PointerValue},
    AddressSpace, IntPredicate, OptimizationLevel,
};
use thiserror::Error;

#[repr(C)]
#[derive(Debug)]
pub struct ProgramResult {
    pub pointer: u16,
    // TODO: do this better
    _padding: [u8; 6],
    pub value: u8,
}

pub struct Codegen<'a> {
    context: &'a Context,
    module: Module<'a>,
    builder: Builder<'a>,
}

impl<'a> Codegen<'a> {
    pub fn from_context(context: &'a Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("brainchuck");
        Self {
            context,
            module,
            builder,
        }
    }

    pub fn generate_code(self, program: &[Command], memory_size: u16) -> Result<String, Error> {
        self.compile_program(program, memory_size)?;
        let ir_code = self.module.print_to_string().to_string();
        Ok(ir_code)
    }

    pub fn run_program(
        self,
        program: &[Command],
        memory_size: u16,
    ) -> Result<ProgramResult, Error> {
        let engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::Aggressive)
            .map_err(Error::JitEngineCreation)?;

        self.compile_program(program, memory_size)?;
        let result = unsafe {
            let main: JitFunction<unsafe extern "C" fn() -> ProgramResult> =
                engine.get_function("main")?;
            main.call()
        };
        Ok(result)
    }

    fn initialize_variables(&self, memory_size: u16) -> Variables<'a> {
        let i8_type = self.context.i8_type();
        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type();
        let memory_type = i8_type.array_type(memory_size.into());

        let cells = self.builder.build_alloca(memory_type, "cells");
        let pointer = self.builder.build_alloca(i16_type, "pointer");
        self.builder
            .build_store(pointer, i16_type.const_int(0, false));

        // Represent void pointers as i8*
        let void_pointer = i8_type.ptr_type(AddressSpace::Generic);
        let memset = self.module.add_function(
            "memset",
            void_pointer.fn_type(
                &[void_pointer.into(), i32_type.into(), i64_type.into()],
                false,
            ),
            Some(Linkage::External),
        );
        let number_elements = i64_type.const_int(memory_size as u64, false);
        self.builder.build_call(
            memset,
            &[
                cells.into(),
                i32_type.const_int(0, false).into(),
                number_elements.into(),
            ],
            "memset_cells",
        );

        Variables { cells, pointer }
    }

    fn initialize_externals(&self) -> Functions<'a> {
        let i32_type = self.context.i32_type();
        let putchar = self.module.add_function(
            "putchar",
            i32_type.fn_type(&[i32_type.into()], false),
            Some(Linkage::External),
        );
        let getchar = self.module.add_function(
            "getchar",
            i32_type.fn_type(&[], false),
            Some(Linkage::External),
        );
        Functions { putchar, getchar }
    }

    fn create_context<'b>(&'b self, memory_size: u16) -> ProgramContext<'b, 'a> {
        let i8_type = self.context.i8_type();
        let i16_type = self.context.i16_type();
        let return_type = self
            .context
            .struct_type(&[i16_type.into(), i8_type.into()], false);
        let main_type = return_type.fn_type(&[], false);
        let main = self.module.add_function("main", main_type, None);
        let basic_block = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(basic_block);

        let variables = self.initialize_variables(memory_size);
        let functions = self.initialize_externals();
        ProgramContext {
            context: self.context,
            builder: &self.builder,
            main,
            variables,
            functions,
        }
    }

    fn compile_program(&self, program: &[Command], memory_size: u16) -> Result<(), Error> {
        let program_context = self.create_context(memory_size);
        program_context.compile_program(program)?;

        let mut state = State::from_context(&program_context);
        state.load_current_cell(&program_context);

        self.builder
            .build_aggregate_return(&[state.pointer.into(), state.cell.unwrap().into()]);

        Ok(())
    }
}

struct State<'ctx> {
    pointer: IntValue<'ctx>,
    cell: Option<IntValue<'ctx>>,
}

impl<'ctx> State<'ctx> {
    fn from_context<'a>(context: &ProgramContext<'a, 'ctx>) -> Self {
        let pointer = context
            .builder
            .build_load(context.variables.pointer, "ptr")
            .into_int_value();
        State {
            pointer,
            cell: None,
        }
    }

    fn store<'a>(&mut self, program_context: &ProgramContext<'a, 'ctx>) {
        self.store_current_cell(program_context);
        program_context
            .builder
            .build_store(program_context.variables.pointer, self.pointer);
    }

    fn store_current_cell<'a>(&mut self, program_context: &ProgramContext<'a, 'ctx>) {
        if let Some(cell) = self.cell {
            let cell_pointer = self.cell_pointer(program_context);
            program_context.builder.build_store(cell_pointer, cell);
            self.cell = None;
        }
    }

    fn load_current_cell<'a>(&mut self, program_context: &ProgramContext<'a, 'ctx>) {
        if self.cell.is_some() {
            return;
        }
        let cell_pointer = self.cell_pointer(program_context);
        let cell = program_context
            .builder
            .build_load(cell_pointer, "cell")
            .into_int_value();
        self.cell = Some(cell);
    }

    fn cell_pointer<'a>(&self, program_context: &ProgramContext<'a, 'ctx>) -> PointerValue<'ctx> {
        let zero = program_context.context.i16_type().const_int(0, false);
        unsafe {
            program_context.builder.build_gep(
                program_context.variables.cells,
                &[zero, self.pointer],
                "cells_pointer",
            )
        }
    }
}

struct Variables<'a> {
    cells: PointerValue<'a>,
    pointer: PointerValue<'a>,
}

struct Functions<'a> {
    putchar: FunctionValue<'a>,
    getchar: FunctionValue<'a>,
}

struct ProgramContext<'a, 'ctx> {
    context: &'ctx Context,
    builder: &'a Builder<'ctx>,
    main: FunctionValue<'ctx>,
    variables: Variables<'ctx>,
    functions: Functions<'ctx>,
}

impl<'a, 'ctx> ProgramContext<'a, 'ctx> {
    fn compile_program(&self, commands: &[Command]) -> Result<(), Error> {
        let state = State::from_context(self);
        self.compile_commands(commands, state)?;
        Ok(())
    }

    fn compile_commands(&self, commands: &[Command], mut state: State<'ctx>) -> Result<(), Error> {
        for command in commands {
            self.compile_command(command, &mut state)?;
        }
        state.store(self);
        Ok(())
    }

    fn compile_command(&self, command: &Command, state: &mut State<'ctx>) -> Result<(), Error> {
        let one = self.context.i8_type().const_int(1, false);
        // TODO: validate
        match command {
            Command::IncrementPointer => {
                state.store_current_cell(self);
                state.pointer = self.builder.build_int_add(state.pointer, one, "ptr")
            }
            Command::DecrementPointer => {
                state.store_current_cell(self);
                state.pointer = self.builder.build_int_sub(state.pointer, one, "ptr");
            }
            Command::IncrementData => {
                state.load_current_cell(self);
                state.cell = Some(self.builder.build_int_add(state.cell.unwrap(), one, "cell"));
            }
            Command::DecrementData => {
                state.load_current_cell(self);
                state.cell = Some(self.builder.build_int_sub(state.cell.unwrap(), one, "cell"));
            }
            Command::Input => {
                let result = self.builder.build_call(
                    self.functions.getchar,
                    &[state.cell.unwrap().into()],
                    "c",
                );
                let read_char = result.try_as_basic_value().left().unwrap().into_int_value();
                state.cell = Some(read_char);
            }
            Command::Output => {
                state.load_current_cell(self);
                self.builder
                    .build_call(self.functions.putchar, &[state.cell.unwrap().into()], "");
            }
            Command::Loop { body } => {
                self.build_loop(body, state)?;
            }
        };
        Ok(())
    }

    fn build_loop(&self, body: &[Command], state: &mut State<'ctx>) -> Result<(), Error> {
        state.store(self);
        let loop_check = self.context.append_basic_block(self.main, "loop_check");
        let loop_body = self.context.append_basic_block(self.main, "loop_body");
        let continuation = self.context.append_basic_block(self.main, "continuation");
        self.builder.build_unconditional_branch(loop_check);

        self.builder.position_at_end(loop_body);
        self.compile_commands(body, State::from_context(self))?;
        self.builder.build_unconditional_branch(loop_check);

        self.builder.position_at_end(loop_check);

        *state = State::from_context(self);
        state.load_current_cell(self);

        let cmp = self.builder.build_int_compare(
            IntPredicate::EQ,
            state.cell.unwrap(),
            self.context.i8_type().const_int(0, false),
            "cmp",
        );
        self.builder
            .build_conditional_branch(cmp, continuation, loop_body);
        self.builder.position_at_end(continuation);
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create JIT engine: {0}")]
    JitEngineCreation(LLVMString),

    #[error("Error looking up function: {0}")]
    FunctionLookup(#[from] FunctionLookupError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_pointer() {
        let context = Context::create();
        let codegen = Codegen::from_context(&context);
        let result = codegen
            .run_program(&[Command::IncrementPointer], 10)
            .unwrap();
        assert_eq!(1, result.pointer);
        assert_eq!(0, result.value);
    }

    #[test]
    fn decrement_pointer() {
        let context = Context::create();
        let codegen = Codegen::from_context(&context);
        let result = codegen
            .run_program(
                &[
                    Command::IncrementPointer,
                    Command::IncrementPointer,
                    Command::DecrementPointer,
                ],
                10,
            )
            .unwrap();
        assert_eq!(1, result.pointer);
        assert_eq!(0, result.value);
    }

    #[test]
    fn increment_data() {
        let context = Context::create();
        let codegen = Codegen::from_context(&context);
        let result = codegen.run_program(&[Command::IncrementData], 10).unwrap();
        assert_eq!(0, result.pointer);
        assert_eq!(1, result.value);
    }

    #[test]
    fn decrement_data() {
        let context = Context::create();
        let codegen = Codegen::from_context(&context);
        let result = codegen.run_program(&[Command::DecrementData], 10).unwrap();
        assert_eq!(0, result.pointer);
        assert_eq!(255, result.value);
    }

    #[test]
    fn program_loop() {
        let context = Context::create();
        let codegen = Codegen::from_context(&context);
        //
        // A dumb tiny snippet to make sure loops work. Equivalent of:
        //
        // ```
        // let mut i = 2; // stored at pointer 0
        // let mut j = 0; // stored at
        // while i > 0 {
        //   j += 1;
        //   i -= 1;
        // }
        // ```
        let result = codegen
            .run_program(
                &[
                    Command::IncrementData,
                    Command::IncrementData,
                    Command::Loop {
                        body: vec![
                            Command::IncrementPointer,
                            Command::IncrementData,
                            Command::DecrementPointer,
                            Command::DecrementData,
                        ],
                    },
                    Command::IncrementPointer,
                ],
                10,
            )
            .unwrap();
        assert_eq!(1, result.pointer);
        assert_eq!(2, result.value);
    }
}
