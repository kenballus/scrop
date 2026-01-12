# scrop

## what's where

### compiler
compiler for a subset of scheme.
emits a stack machine assembly language.

rust.

### assembler
assembler for the stack machine assembly language.
emits bytecode.

python.

### interpreter
interpreter for the bytecode.
basically just a rop executor.

c and x86\_64 asm.

## building

(tested only on x86\_64 linux)

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
