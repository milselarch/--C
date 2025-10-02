# `--C`

Goals of this programming language:
1. Syntax supported is a subset of C
   1. Any additional syntax that isn't supported by the C spec should
        come with support for transpilation to C
   2. An exception to this is support for infinite length integers
2. It can compile to x86-64 assembly
3. It can compile down to cellular automata
    1. There should be a fixed position or range of positions from
       which it can be determined whether the program has halted
4. True support for infinite length integers
   1. This is as opposed to arbitrary length integers with a length that is
      bounded by the largest int datatype
   2. More specifically the infinite length integer should theoretically be able to 
      actually grow to infinity assuming a machine with an infinite address space 
      and infinite register size, but where the data size at each address 
      is finite still


# Setup
Rust is required to build the compiler.  
Pull the C compiler tests submodule with `git submodule update --init --recursive`
1. `cargo build --release`
2. `./target/release/ca-compiler <YOUR_C_FILE.c>`

## Examples

Test compilation on all test cases in chapter 2:
```bash
cargo build && \
writing-a-c-compiler-tests/test_compiler ./target/debug/ca-compiler --chapter 2
```

Test asm gen on all test cases in chapter 2:
```bash
cargo build && \
writing-a-c-compiler-tests/test_compiler ./target/debug/ca-compiler --chapter 2 --stage codegen
```

Supported stages:
- lex
- parse
- tacky
- codegen