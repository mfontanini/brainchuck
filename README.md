# brainchuck

A LLVM based JIT brainfuck compiler/interpreter

# What?

This uses [inkwell](https://github.com/TheDan64/inkwell) to transform a brainfuck program into LLVM IR code and then
JIT execute it. This in turn makes this interpreter run quite fast.

# Why?

I discovered inkwell and wanted to play around with it. After writing [brainrust](https://github.com/mfontanini/brainrust)
this seemed like the next logical step.

# Usage

Build, run, and provide the source file as a parameter, or use `-` to read the program from stdin. e.g.

```brainfuck
$ cargo run -- -
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/brainchuck -`
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.

Hello World!
$
```

If you'd like to see the generated IR code rather than execute the program, use `-p`. e.g.

```llvm
$ cargo run -- -p -
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/brainchuck -p -`
>>>+.
; ModuleID = 'brainchuck'
source_filename = "brainchuck"

define { i16, i8 } @main() {
entry:
  %cells = alloca [30000 x i8], align 1
  %pointer = alloca i16, align 2
  store i16 0, i16* %pointer, align 2
  %memset_cells = call i8* @memset([30000 x i8]* %cells, i32 0, i64 30000)
  %ptr = load i16, i16* %pointer, align 2
  %ptr1 = add i16 %ptr, i8 1
  %ptr2 = add i16 %ptr1, i8 1
  %ptr3 = add i16 %ptr2, i8 1
  %cells_pointer = getelementptr [30000 x i8], [30000 x i8]* %cells, i16 0, i16 %ptr3
  %cell = load i8, i8* %cells_pointer, align 1
  %cell4 = add i8 %cell, 1
  %0 = call i32 @putchar(i8 %cell4)
  %cells_pointer5 = getelementptr [30000 x i8], [30000 x i8]* %cells, i16 0, i16 %ptr3
  store i8 %cell4, i8* %cells_pointer5, align 1
  store i16 %ptr3, i16* %pointer, align 2
  %ptr6 = load i16, i16* %pointer, align 2
  %cells_pointer7 = getelementptr [30000 x i8], [30000 x i8]* %cells, i16 0, i16 %ptr6
  %cell8 = load i8, i8* %cells_pointer7, align 1
  %mrv = insertvalue { i16, i8 } undef, i16 %ptr6, 0
  %mrv9 = insertvalue { i16, i8 } %mrv, i8 %cell8, 1
  ret { i16, i8 } %mrv9
}

declare i8* @memset(i8*, i32, i64)

declare i32 @putchar(i32)

declare i32 @getchar()

$
```

# Notes

## Very large programs crash

It seems like for some unknown reason if the source program is very large, the external functions used here (e.g. `putchar`,
`getchar` and `memset`) are not correctly linked, leading to segmentation faults. This can be fixed by using PIC relocation
mode but I can't seem to be able to enable that in inkwell.

The only program that I've found so far that fails because of this is [this one](https://github.com/kostya/benchmarks/blob/master/brainfuck/mandel.b).

## x64 only

This potentially only works in x64 as there's a couple of assumptions on integer widths I'm making that would likely
not work on other architectures.

## LLVM version

This currently targets llvm 11.
