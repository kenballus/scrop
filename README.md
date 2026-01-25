# scrop

only tested on x86\_64 linux with clang 21.1.6.
almost certainly doesn't work with anything else.

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
bytecode opcodes are the addresses of their implementations in the interpreter.
in other words, the bytecode programs are ropchains for the interpreter.

x86\_64 asm and a little bit of c without any libc.

## building

```sh
./build.bash
```

## example

```sh
printf '(if (= 10 (+ 1 2 3 4)) (integer->char 97) (integer->char 65))' | ./run.bash
```
```
#\a
```
