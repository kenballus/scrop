# scrop

## what's where

### compiler
A compiler for a subset of Scheme.
Emits a stack machine assembly language.

Rust.

### assembler
An assembler for the stack machine assembly language.
Emits bytecode.

Python.

### interpreter
An interpreter for the bytecode. Basically just a ROP executor.

C and x86\_64 asm.

## building

```sh
cd interpreter \
    && make \
    && cd .. \
    && cd compiler \
    && cargo build \
    && cd ..
```

## example

```sh
printf '(integer->char (+ 1 (char->integer #\\a)))' \
    | ./compiler/target/debug/compiler \
    | uv run assembler/main.py \
    | ./interpreter/interpreter
```
```
#\b
```
