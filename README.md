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

x86\_64 asm and a little bit of c (without libc).

## building

```sh
./build.bash
```

## example

```sh
cat <<EOF | ./run.bash
(string
    (let ((sum (lambda (l) (fold + 0 l))))
        (integer->char (sum (list 1 2 3 4 5 6 7 8 9 10 11))))
    #\N
    #\G
    #\U
    #\L
    #\A
    #\R)
EOF
```
```
"BNGULAR"
```

## manual

### data types
- integer
  - represents the range $[-2^{61}, 2^{61} - 1]$
  - e.g., `-1`
- character
  - represents the ascii range.
  - e.g., `#\a`
- bool
  - good ol' boolean
  - e.g., `#f`
- cons
  - represents a pair of objects.
  - e.g., `(cons 1 '())`
- lambda
  - represents a function.
  - e.g., `(lambda (x) (+ x 1))`
- vector
  - represents a list that isn't slow.
  - e.g., `(vector #\b #\n #\g #\u #\l #\a #\r)`
- string
  - a vector, but special-cased for chars to not waste space
  - e.g., `"bngular"`
- null
  - special value that means the end of a list
  - `'()`
- unspecified
  - special value that is the result of evaluating an expression with no result
  - e.g., `(begin)`

### built-in lambdas
- `(zero? x)`
  - returns whether `x` is zero.
- `(integer? x)`
  - returns whether `x` is an integer.
- `(boolean? x)`
  - returns whether `x` is a boolean.
- `(char? x)`
  - returns whether `x` is a character.
- `(null? x)`
  - returns whether `x` is `'()`.
- `(not x)`
  - returns `#t` if `x` is `#f`, and `#f` otherwise.
- `(char->integer x)`
  - converts `x`, which must be an integer in the appropriate range, to a character.
- `(+ ...args)`
  - sums `args`, which must be integers.
- `(- ...args)`
  - subtracts `args`, which must be integers.
- `(* ...args)`
  - multiplies `args`, which must be integers.
- `(< ...args)`
  - returns whether `args`, which must be integers, are strictly increasing.
- `(= ...args)`
  - returns whether `args`, which must be integers, are equal.
- `(eq? ...args)`
  - returns whether everything in `args` is pointer-equal. for objects that are not heap-allocated, they are pointer-equal if they hold the same value. for objects that are heap-allocated, they are pointer-equal if they are the same object.
- `(string ...args)`
  - constructs a string by concatenating `args`, which must be characters.
- `(string-append ...args)`
  - constructs a string by concatenating `args`, which must be strings.
- `(string-ref s n)`
  - returns the `n`th character from `s`.
- `(string-set! s n c)`
  - sets the `n`th character of `s` to `c`.
- `(vector ...args)`
  - constructs a vector containing each of `args`.
- `(vector-ref v n)`
  - returns the `n`th character from `v`.
- `(vector-set! v n x)`
  - sets the `n`th item of `v` to `x`.
- `(cons first second)`
  - makes a `cons` containing `first` and `second`.
- `(car l)`
  - returns the first thing in `l`, which must be a `cons`.
- `(cdr l)`
  - returns the second thing in `l`, which must be a `cons`.
- `list args`
  - returns `args`
- `map (f l)`
  - maps `f`, which must be a lambda, over `l`, which must be a list.
- `fold (f i l)`
  - folds `f`, which must be a lambda, over `l`, which must be a list, with initial value `i`.
- `foldr (f i l)`
  - folds `f`, which must be a lambda, over `l`, which must be a list, with initial value `i`, right-associative.
- `reverse (l)`
  - reverses `l`, which must be a list.

### special forms
- `(let (...bindings) ...statements)`
  - for each item in `bindings`, which must be a list of `cons`es in which the first element is a symbol, binds each of these symbols to the result of evaluating the second element in its `cons`, then evaluates each statement in `statements` in a new environment that includes the fresh bindings.
  - e.g., `(let ((a 1) (b 2)) (+ a b))`
- `(let* (...bindings) ...statements)`
  - the same as `let`, but bindings are permitted to use previous bindings.
  - e.g., `(let* ((a 1) (b a)) b)`
- `(begin ...statements)`
  - the same as `let`, but no bindings.
- `(apply f (...args))`
  - calls `f` with the arguments in `args`, which must be a list.
- `(if cond conseq alt)`
  - evaluates `cond`, executes `alt` if it resulted in `#f`, and `conseq` otherwise.
- `(if cond conseq)`
  - evaluates `cond`, executes `conseq` if it did not result in `#f`, and returns `#<unspecified>` otherwise.
- `(lambda (...args) ...statements)`
  - makes a lambda.
  - e.g., `(lambda (n) (+ n 1))`
- `(lambda (...args . rest) ...statements)`
  - makes a variadic lambda with some required arguments.
  - e.g., `(lambda (head . rest) rest)`
- `(lambda args ...statements)`
  - makes a variadic lambda with no required arguments
  - e.g., `(lambda args (car args))`
- `(lambdarec name ...)`
  - exactly like `lambda`, but takes an extra argument `name`, which is bound to the resulting lambda when it is executed. allows for recursive lambdas.
  - e.g., `(lambdarec fact (n) (* n (fact (- n 1))))`
