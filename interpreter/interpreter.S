.intel_syntax noprefix

#include "constants.h"

#define vm_pc rsp
#define vm_sp rbx
#define vm_hp rbp
#define vm_fp r12
#define vm_lr r13

#define code_base r15

// Each instruction is a 64-bit opcode and a maybe-unused 64-bit immediate.
// Each opcode is simply the address of its implementation.
// (See the linker script for these addresses)
// Each immediate is tagged with its type in its low bits as follows:
// -       00 => int
// - 00001111 => char
// -  0011111 => bool
// - 00101111 => '()
// -   all 1s => #<unspecified>
// -      001 => pair
// -      011 => string
// -      010 => vector
// -      110 => lambda

// Needs to preserve flags
#define PUSH(X) \
    lea vm_sp, [vm_sp - STACK_SLOT_SIZE] ; \
    mov qword ptr [vm_sp], X

#define GET_IMMEDIATE(R64) \
    mov R64, qword ptr [vm_pc - IMMEDIATE_SIZE]

// Needs to preserve flags
#define POP(X) \
    mov X, qword ptr [vm_sp] ; \
    lea vm_sp, [vm_sp + STACK_SLOT_SIZE]

// Needs to preserve flags
#define DROP \
    lea vm_sp, [vm_sp + STACK_SLOT_SIZE]

// Needs to preserve flags
#define PEEK(X, I) \
    mov X, qword ptr [vm_sp + I * STACK_SLOT_SIZE]

#define JMP_IF_NULL(R64, LABEL) \
    cmp R64, TAGGED_NULL ; \
    je LABEL

#define JMP_IF_NOT_INT(R64, LABEL) \
    PUSH(R64) ; \
    and R64, INT_MASK ; \
    test R64, R64 ; \
    POP(R64) ; \
    jne LABEL

#define JMP_IF_BOOL(R64, LABEL) \
    cmp R64, TRUE ; \
    je LABEL ; \
    cmp R64, FALSE ; \
    je LABEL

#define JMP_IF_CHAR(R64, LABEL) \
    PUSH(R64) ; \
    and R64, CHAR_MASK ; \
    cmp R64, CHAR_SUFFIX ; \
    POP(R64) ; \
    PUSH(R64) ; \
    cmovne R64, qword ptr [rip + zero_location] ; \
    sub R64, TAGGED_CHAR_MIN ; \
    cmp R64, TAGGED_CHAR_MAX - TAGGED_CHAR_MIN ; \
    POP(R64) ; \
    jbe LABEL

#define JMP_IF_PAIR(R64, LABEL) \
    PUSH(R64) ; \
    and R64, PAIR_MASK ; \
    cmp R64, PAIR_SUFFIX ; \
    POP(R64) ; \
    je LABEL

#define JMP_IF_NOT_PAIR(R64, LABEL) \
    PUSH(R64) ; \
    and R64, PAIR_MASK ; \
    cmp R64, PAIR_SUFFIX ; \
    POP(R64) ; \
    jne LABEL

#define JMP_IF_NOT_STRING(R64, LABEL) \
    PUSH(R64) ; \
    and R64, STRING_MASK ; \
    cmp R64, STRING_SUFFIX ; \
    POP(R64) ; \
    jne LABEL

#define JMP_IF_NOT_VECTOR(R64, LABEL) \
    PUSH(R64) ; \
    and R64, VECTOR_MASK ; \
    cmp R64, VECTOR_SUFFIX ; \
    POP(R64) ; \
    jne LABEL

#define JMP_IF_NOT_LAMBDA(R64, LABEL) \
    PUSH(R64) ; \
    and R64, LAMBDA_MASK ; \
    cmp R64, LAMBDA_SUFFIX ; \
    POP(R64) ; \
    jne LABEL

#define TAG_INT(R64) \
    shl R64, 2

#define TAG_CONST_INT(V) \
    V << 2

#define UNTAG_INT(R64) \
    sar R64, 2

#define TAG_BOOL(R64) \
    shl R64, 7 ; \
    or R64, BOOL_SUFFIX

#define UNTAG_BOOL(R64) \
    shr R64, 7

#define UNTAG_CHAR(R64) \
    shr R64, 8

#define TAG_CHAR(R64) \
    shl R64, 8 ; \
    or R64, CHAR_SUFFIX

#define TAG_PAIR(R64) \
    or R64, 1

#define UNTAG_PAIR(R64) \
    and R64, -2

#define TAG_STRING(R64) \
    or R64, STRING_SUFFIX

#define UNTAG_STRING(R64) \
    and R64, -4

#define TAG_VECTOR(R64) \
    or R64, VECTOR_SUFFIX

#define UNTAG_VECTOR(R64) \
    and R64, -4

#define TAG_LAMBDA(R64) \
    or R64, LAMBDA_SUFFIX

#define UNTAG_LAMBDA(R64) \
    and R64, ~LAMBDA_MASK

#define UNPACK(T, DST, ERR) \
    POP(DST) ; \
    JMP_IF_NOT_##T(DST, ERR) ; \
    UNTAG_##T(DST)

#define PACK(T, SRC) \
    TAG_##T(SRC) ; \
    PUSH(SRC)

#define NEXT_INSTRUCTION \
    ret IMMEDIATE_SIZE

.section .text

die:
    ud2

zero_location:
    .quad 0

.global interpret
interpret:
    mov vm_hp, rsp   // vm heap pointer
    mov vm_pc, rdi   // vm instruction pointer
    mov vm_sp, rsi   // vm stack pointer
    xor vm_fp, vm_fp // vm frame pointer
    xor vm_lr, vm_lr // vm frame pointer
    mov code_base, vm_pc // code_base
    NEXT_INSTRUCTION

.section .text.add
.global add
add:
    UNPACK(INT, rdi, die) // arity
    test rdi, rdi
    jne 1f
    PUSH(TAG_CONST_INT(0))
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rax, die)
2:
    // base case
    cmp rdi, 1
    jne 1f
    PACK(INT, rax)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rcx, die)
    add rax, rcx
    dec rdi
    jmp 2b

.section .text.sub
.global sub
sub:
    UNPACK(INT, rdi, die) // arity
    test rdi, rdi
    jne 1f
    PUSH(0)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rax, die)
1:
    cmp rdi, 1
    jne 1f
    // unary special case
    neg rax
    PACK(INT, rax)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rcx, die)
    sub rax, rcx
    dec rdi
    cmp rdi, 1
    jne 1b
    PACK(INT, rax)
    NEXT_INSTRUCTION

.section .text.mul
.global mul
mul:
    UNPACK(INT, rdi, die) // arity
    test rdi, rdi
    jne 1f
    PUSH(TAG_CONST_INT(1))
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rax, die)
2:
    // base case
    cmp rdi, 1
    jne 1f
    PACK(INT, rax)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rcx, die)
    mul rcx
    dec rdi
    jmp 2b


.section .text.lt
.global lt
lt:
    UNPACK(INT, rdi, die) // arity
    mov esi, 1 // result
    test rdi, rdi
    jne 1f
    // 0 args
    PUSH(TRUE)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rax, die)
    dec rdi
    test rdi, rdi
    jne 1f
    // 1 arg
    PUSH(TRUE)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rcx, die)
    cmp rax, rcx
    setl al
    and sil, al
    mov rax, rcx
    dec rdi
    test rdi, rdi
    jne 1b
    PACK(BOOL, rsi)
    NEXT_INSTRUCTION

.section .text.eq
.global eq
eq:
    UNPACK(INT, rdi, die) // arity
    mov esi, 1 // result
    test rdi, rdi
    jne 1f
    // 0 args
    PUSH(TRUE)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rax, die)
    dec rdi
    test rdi, rdi
    jne 1f
    // 1 arg
    PUSH(TRUE)
    NEXT_INSTRUCTION
1:
    UNPACK(INT, rcx, die)
    cmp rax, rcx
    sete al
    and sil, al
    mov rax, rcx
    dec rdi
    test rdi, rdi
    jne 1b
    PACK(BOOL, rsi)
    NEXT_INSTRUCTION

.section .text.eqp
.global eqp
eqp:
    UNPACK(INT, rdi, die) // arity
    mov esi, 1 // result
    test rdi, rdi
    jne 1f
    // 0 args
    PUSH(TRUE)
    NEXT_INSTRUCTION
1:
    POP(rax)
    dec rdi
    test rdi, rdi
    jne 1f
    // 1 arg
    PUSH(TRUE)
    NEXT_INSTRUCTION
1:
    POP(rcx)
    cmp rax, rcx
    sete al
    and sil, al
    mov rax, rcx
    dec rdi
    test rdi, rdi
    jne 1b
    PACK(BOOL, rsi)
    NEXT_INSTRUCTION

.section .text.load
.global load
load:
    GET_IMMEDIATE(rax)
    PUSH(rax)
    NEXT_INSTRUCTION

.section .text.zerop
.global zerop
zerop:
    UNPACK(INT, rax, die)
    test rax, rax
    mov eax, 0 // cannot be xor because of flags
    sete al
    PACK(BOOL, rax)
    NEXT_INSTRUCTION

.section .text.integerp
.global integerp
integerp:
    POP(rax)
    JMP_IF_NOT_INT(rax, 1f)
    PUSH(TRUE)
    NEXT_INSTRUCTION
1:
    PUSH(FALSE)
    NEXT_INSTRUCTION

.section .text.booleanp
.global booleanp
booleanp:
    POP(rax)
    JMP_IF_BOOL(rax, 1f)
    PUSH(FALSE)
    NEXT_INSTRUCTION
1: // bool
    PUSH(TRUE)
    NEXT_INSTRUCTION

.section .text.charp
.global charp
charp:
    POP(rax)
    JMP_IF_CHAR(rax, 1f)
    PUSH(FALSE)
    NEXT_INSTRUCTION
1: // char
    PUSH(TRUE)
    NEXT_INSTRUCTION

.section .text.nullp
.global nullp
nullp:
    POP(rax)
    JMP_IF_NULL(rax, 1f)
    PUSH(FALSE)
    NEXT_INSTRUCTION
1: // '()
    PUSH(TRUE)
    NEXT_INSTRUCTION

.section .text.not
.global not
not:
    POP(rax)
    JMP_IF_BOOL(rax, 1f)
    PUSH(FALSE)
    NEXT_INSTRUCTION
1: // bool
    UNTAG_BOOL(rax)
    not rax
    and rax, 1
    PACK(BOOL, rax)
    NEXT_INSTRUCTION

.section .text.chartoint
.global chartoint
chartoint:
    POP(rax)
    JMP_IF_CHAR(rax, 1f)
    jmp die
1: // char
    UNTAG_CHAR(rax)
    PACK(INT, rax)
    NEXT_INSTRUCTION

.section .text.inttochar
.global inttochar
inttochar:
    UNPACK(INT, rax, die)
    cmp rax, 128
    jae die
    PACK(CHAR, rax)
    NEXT_INSTRUCTION

.section .text.cjump
.global cjump
cjump:
    GET_IMMEDIATE(rax)
    POP(rcx)
    JMP_IF_BOOL(rcx, 1f)
    jmp die
1:
    cmp rcx, FALSE
    je 1f
    shl rax, 4
    add vm_pc, rax
1:
    NEXT_INSTRUCTION

.section .text.get
.global get
get:
    GET_IMMEDIATE(rax) // The offset from the sp to get
    mov rax, qword ptr [vm_sp + rax * STACK_SLOT_SIZE]
    PUSH(rax)
    NEXT_INSTRUCTION

.section .text.forget
.global forget
forget:
    DROP
    NEXT_INSTRUCTION

.section .text.jump
.global jump
jump:
    GET_IMMEDIATE(rax)
    shl rax, 4
    add vm_pc, rax
    NEXT_INSTRUCTION

.section .text.cons
.global cons
cons:
    POP(rax) // cdr
    POP(rcx) // car
    sub vm_hp, 16 // alloc(16)
    mov rdi, vm_hp
    mov qword ptr [rdi], rcx
    mov qword ptr [rdi + 8], rax
    PACK(PAIR, rdi)
    NEXT_INSTRUCTION

.section .text.car
.global car
car:
    POP(rax)
    JMP_IF_PAIR(rax, 1f)
    jmp die
1:
    UNTAG_PAIR(rax)
    mov rax, qword ptr [rax]
    PUSH(rax)
    NEXT_INSTRUCTION

.section .text.cdr
.global cdr
cdr:
    POP(rax)
    JMP_IF_PAIR(rax, 1f)
    jmp die
1:
    UNTAG_PAIR(rax)
    mov rax, qword ptr [rax + 8]
    PUSH(rax)
    NEXT_INSTRUCTION

.section .text.string
.global string
string:
    UNPACK(INT, rdi, die) // arity
    // alloc(UP_TO_8(arity) + 8)
    sub vm_hp, rdi
    and vm_hp, -8
    sub vm_hp, 8
    mov rax, vm_hp // result
    mov qword ptr [vm_hp], rdi // write arity into the struct
    xor esi, esi
2:
    test rdi, rdi
    jne 1f
    PACK(STRING, rax)
    NEXT_INSTRUCTION
1:
    POP(rcx)
    JMP_IF_CHAR(rcx, 1f)
    jmp die
1:
    UNTAG_CHAR(rcx)
    mov byte ptr [rax + rsi + 8], cl
    inc rsi
    dec rdi
    jmp 2b

.section .text.stringref
.global stringref
stringref:
    UNPACK(INT, rcx, 1f) // index
    UNPACK(STRING, rax, 1f) // index
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f
    mov al, byte ptr [rax + rcx + 8]
    and rax, 0xff
    PACK(CHAR, rax)
    NEXT_INSTRUCTION
1:
    ud2

.section .text.stringset
.global stringset
stringset:
    POP(rsi) // char
    JMP_IF_CHAR(rsi, 1f)
    ud2
1:
    UNTAG_CHAR(rsi)
    UNPACK(INT, rcx, 1f) // index
    UNPACK(STRING, rax, 1f) // string
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f // index out of bounds
    mov byte ptr [rax + rcx + 8], sil
    and rax, 0xff
    PACK(CHAR, rax)
    NEXT_INSTRUCTION
1:
    ud2

.section .text.stringappend
.global stringappend
stringappend:
    UNPACK(INT, rdx, die) // arity
    test rdx, rdx
    jne 1f
    // no args
    sub vm_hp, 8
    mov qword ptr [vm_hp], 0
    mov rax, vm_hp
    PACK(STRING, rax)
    NEXT_INSTRUCTION
1:
    // at least one arg
    xor ecx, ecx // counter
    xor eax, eax // length
1:
    cmp rcx, rdx
    je 1f
    PEEK(rsi, rcx) // args[rcx]
    JMP_IF_NOT_STRING(rsi, die)
    UNTAG_STRING(rsi)
    add rax, qword ptr [rsi]
    inc rcx
    jmp 1b
1:
    // alloc(sum_of_strlens + 8)
    sub vm_hp, rax
    and vm_hp, -8
    sub vm_hp, 8
    mov qword ptr [vm_hp], rax // result length
    lea rdi, [vm_hp + 8] // result buffer
1:
    test rdx, rdx
    je 1f
    UNPACK(STRING, rsi, die) // arg
    mov rcx, qword ptr [rsi] // count
    add rsi, 8
    rep movsb
    dec rdx
    jmp 1b
1:
    mov rax, vm_hp
    PACK(STRING, rax)
    NEXT_INSTRUCTION

.section .text.vector
.global vector
vector:
    UNPACK(INT, rdi, die) // arity
    // alloc(arity * 8 + 8)
    neg rdi
    lea vm_hp, [vm_hp + rdi * 8 - 8]
    neg rdi
    mov rax, vm_hp // result
    mov qword ptr [vm_hp], rdi // write arity into the struct
    xor esi, esi
2:
    test rdi, rdi
    jne 1f
    PACK(VECTOR, rax)
    NEXT_INSTRUCTION
1:
    POP(rcx)
    mov qword ptr [rax + rsi * 8 + 8], rcx
    inc rsi
    dec rdi
    jmp 2b

.section .text.vectorref
.global vectorref
vectorref:
    UNPACK(INT, rcx, die) // index
    UNPACK(VECTOR, rax, die) // vector
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f
    mov rax, qword ptr [rax + rcx * 8 + 8]
    PUSH(rax)
    NEXT_INSTRUCTION

.section .text.vectorset
.global vectorset
vectorset:
    POP(rsi) // arg
    UNPACK(INT, rcx, die) // index
    UNPACK(VECTOR, rax, die) // vector
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f // index out of bounds
    mov qword ptr [rax + rcx * 8 + 8], rsi
    PUSH(rax)
    NEXT_INSTRUCTION

.section .text.vectorappend
.global vectorappend
vectorappend:
    UNPACK(INT, rdx, die) // arity
    test rdx, rdx
    jne 1f
    // no args
    sub vm_hp, 8
    mov qword ptr [vm_hp], 0
    mov rax, vm_hp
    PACK(VECTOR, rax)
    NEXT_INSTRUCTION
1:
    // at least one arg
    xor ecx, ecx // counter
    xor eax, eax // length
1:
    cmp rcx, rdx
    je 1f
    PEEK(rsi, rcx) // args[rcx]
    JMP_IF_NOT_VECTOR(rsi, die)
    UNTAG_VECTOR(rsi)
    add rax, qword ptr [rsi]
    inc rcx
    jmp 1b
1:
    // alloc(sum_of_veclens * 8 + 8)
    neg rax
    lea vm_hp, [vm_hp + rax * 8 - 8]
    neg rax
    mov qword ptr [vm_hp], rax // result length
    lea rdi, [vm_hp + 8] // result buffer
1:
    test rdx, rdx
    je 1f
    UNPACK(VECTOR, rsi, die) // arg
    mov rcx, qword ptr [rsi] // count
    add rsi, 8
    rep movsq
    dec rdx
    jmp 1b
1:
    mov rax, vm_hp
    PACK(VECTOR, rax)
    NEXT_INSTRUCTION

.section .text.tailcall
.global tailcall
tailcall:
    // saved vm_fp <----- vm_fp
    // prev locals
    // prev lambda
    // saved lr
    // args
    // lambda
    // arity       <----- vm_sp

    UNPACK(INT, rdi, die) // num args passed
    UNPACK(LAMBDA, rax, die) // lambda
    mov rdx, qword ptr [rax + 16] // arity
    cmp rdx, 0
    jge 2f
    mov rcx, rdi // rcx = # of passed args
    mov rsi, rdx // rsi = # of required args
    not rsi
    sub rcx, rsi
    cmp rcx, 0
    jl die // too few args passed
    mov rsi, TAGGED_NULL
1:
    PUSH(rsi)
    cmp rcx, 0
    je 1f
    POP(rsi)
    POP(r8)
    sub vm_hp, 16 // alloc(16)
    mov qword ptr [vm_hp], r8 // car
    mov qword ptr [vm_hp + 8], rsi // cdr
    mov rsi, vm_hp
    TAG_PAIR(rsi)
    dec rcx
    jmp 1b
1:
    neg rdx
    jmp 1f
2:
    cmp rdi, rdx
    jne die // arity mismatch
1:
    // saved vm_fp <----- vm_fp
    // prev locals
    // prev lambda
    // saved lr
    // args        <----- vm_sp

    mov r8, qword ptr [vm_sp + rdx * STACK_SLOT_SIZE] // saved lr
    mov rcx, rdx // arity
1:
    test rcx, rcx
    je 1f
    mov rdi, qword ptr [vm_sp + rcx * STACK_SLOT_SIZE - STACK_SLOT_SIZE]
    mov rsi, rdx
    sub rsi, rcx
    neg rsi
    mov qword ptr [rsi * STACK_SLOT_SIZE + vm_fp - STACK_SLOT_SIZE], rdi
    dec rcx
    jmp 1b
1:
    neg rdx
    lea vm_sp, [vm_fp + rdx * STACK_SLOT_SIZE]
    neg rdx
    // saved vm_fp <----- vm_fp
    // args        <----- vm_sp

    // unpack freevars to stack
    mov rdx, qword ptr [rax + 8] // freevar vector
    mov rcx, qword ptr [rdx] // vector length
1:
    test rcx, rcx
    je 1f
    mov rsi, qword ptr [rdx + rcx * 8]
    PUSH(rsi)
    dec rcx
    jmp 1b
1:
    mov rdi, rax
    PACK(LAMBDA, rdi) // the lambda
    PUSH(r8) // lr from the caller's frame
    mov vm_pc, qword ptr [rax] // code offset
    shl vm_pc, 4
    add vm_pc, code_base

    NEXT_INSTRUCTION

.section .text.call
.global call
call:
    UNPACK(INT, rdi, die) // num args passed
    UNPACK(LAMBDA, rax, die) // lambda
    mov rdx, qword ptr [rax + 16] // arity
    cmp rdx, 0
    jge 2f
    mov rcx, rdi // rcx = # of passed args
    mov rsi, rdx // rsi = # of required args
    not rsi
    sub rcx, rsi
    cmp rcx, 0
    jl die // too few args passed
    mov rsi, TAGGED_NULL
1:
    PUSH(rsi)
    cmp rcx, 0
    je 1f
    POP(rsi)
    POP(r8)
    sub vm_hp, 16 // alloc(16)
    mov qword ptr [vm_hp], r8 // car
    mov qword ptr [vm_hp + 8], rsi // cdr
    mov rsi, vm_hp
    TAG_PAIR(rsi)
    dec rcx
    jmp 1b
1:
    neg rdx
    jmp 1f
2:
    cmp rdi, rdx
    jne die // arity mismatch
1:

    // unpack freevars to stack
    mov rdx, qword ptr [rax + 8] // freevar vector
    mov rcx, qword ptr [rdx] // vector length
1:
    test rcx, rcx
    je 1f
    mov rsi, qword ptr [rdx + rcx * 8]
    PUSH(rsi)
    dec rcx
    jmp 1b

1:
    mov rdi, rax
    PACK(LAMBDA, rdi) // the lambda
    PUSH(vm_lr)
    mov vm_lr, vm_pc
    mov vm_pc, qword ptr [rax] // code offset
    shl vm_pc, 4
    add vm_pc, code_base

    // the stack now looks like this:
    // saved vm_fp    <----- vm_fp
    // arg 0
    // ...
    // arg N
    // freevar N
    // ...
    // freevar 0
    // lambda
    // saved vm_lr    <----- vm_sp
    NEXT_INSTRUCTION

.section .text.apply
.global apply
apply:
    POP(rcx)
    UNPACK(LAMBDA, rax, die) // the lambda
    PUSH(vm_fp)
    mov vm_fp, vm_sp
    PUSH(rcx)
    xor ecx, ecx
1:
    cmp qword ptr [vm_sp], TAGGED_NULL
    je 1f
    UNPACK(PAIR, rsi, die)
    mov rdi, qword ptr [rsi]
    mov rsi, qword ptr [rsi + 8]
    PUSH(rdi)
    PUSH(rsi)
    inc rcx
    jmp 1b
1:
    DROP
    PACK(LAMBDA, rax)
    PACK(INT, rcx)
    jmp call

.section .text.tailapply
.global tailapply
tailapply:
    POP(rcx)
    UNPACK(LAMBDA, rax, die) // the lambda
    PUSH(rcx)
    xor ecx, ecx
1:
    cmp qword ptr [vm_sp], TAGGED_NULL
    je 1f
    UNPACK(PAIR, rsi, die)
    mov rdi, qword ptr [rsi]
    mov rsi, qword ptr [rsi + 8]
    PUSH(rdi)
    PUSH(rsi)
    inc rcx
    jmp 1b
1:
    DROP
    PACK(LAMBDA, rax)
    PACK(INT, rcx)
    jmp tailcall

.section .text.primapply
.global primapply
primapply:
    GET_IMMEDIATE(rdx)
    PEEK(rsi, 2) // grab the arg list
    PUSH(rsi)
    xor ecx, ecx
1:
    // NOTE: this assumes that all variadic primitives
    // accept 0 or more arguments.
    // If you ever add a variadic primitive that has a nonzero
    // minimum arity, this will not work.
    cmp qword ptr [vm_sp], TAGGED_NULL
    je 1f
    UNPACK(PAIR, rsi, die)
    mov rdi, qword ptr [rsi]
    mov rsi, qword ptr [rsi + 8]
    PUSH(rdi)
    PUSH(rsi)
    inc rcx
    jmp 1b
1:
    DROP
    PACK(INT, rcx)
    jmp rdx

.section .text.lambda
.global lambda
lambda:
    GET_IMMEDIATE(rdx) // code offset
    UNPACK(INT, rcx, die) // arity
    UNPACK(VECTOR, rax, die) // freevar value vector
    sub vm_hp, 24
    mov qword ptr [vm_hp], rdx
    mov qword ptr [vm_hp + 8], rax
    mov qword ptr [vm_hp + 16], rcx
    mov rax, vm_hp
    PACK(LAMBDA, rax)
    NEXT_INSTRUCTION

.section .text.return
.global return
return:
    POP(rdx) // retval
    mov vm_pc, vm_lr
    POP(vm_lr) // lr

    mov vm_sp, vm_fp
    POP(vm_fp)
    PUSH(rdx)
    NEXT_INSTRUCTION

.section .text.frame
.global frame
frame:
    PUSH(vm_fp)
    mov vm_fp, vm_sp
    NEXT_INSTRUCTION

.section .text.done
.global done
done:
    test vm_fp, vm_fp
    jne die // not in the base frame
    mov rdi, qword ptr [vm_sp] // Grab what's on top of the stack
    mov rsp, vm_hp          // Put the stack back
    and rsp, -16             // Align the stack for a real function call
    call print_value_and_exit  // Call back into C
