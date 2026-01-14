# scrop

(this works only on x86\_64 linux)

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

x86\_64 asm and a little bit of c.

## building

```sh
./build.bash
```

## example

```sh
printf '(if (= 10 (+ 1 2 3 4)) (integer->char 97) (integer->char 65))' \
    | ./compiler/target/debug/compiler \
    | uv run assembler/main.py \
    | ./interpreter/interpreter
```

### stdout
```
#\a
```

### stderr
```
&ast = [
    If(
        [
            PrimitiveFnCall(
                Eq,
                [
                    Int(
                        10,
                    ),
                    PrimitiveFnCall(
                        Add,
                        [
                            Int(
                                1,
                            ),
                            Int(
                                2,
                            ),
                            Int(
                                3,
                            ),
                            Int(
                                4,
                            ),
                        ],
                    ),
                ],
            ),
            PrimitiveFnCall(
                IntToChar,
                [
                    Int(
                        97,
                    ),
                ],
            ),
            PrimitiveFnCall(
                IntToChar,
                [
                    Int(
                        65,
                    ),
                ],
            ),
        ],
    ),
]
```
