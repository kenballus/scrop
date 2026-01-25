.intel_syntax noprefix

#include "constants.h"

#define vm_pc rsp
#define vm_sp rbx
#define vm_hp rbp
#define stack_slots_used r12

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

// Needs to preserve flags
#define PUSH(X) \
    lea vm_sp, [vm_sp - STACK_SLOT_SIZE] ; \
    mov qword ptr [vm_sp], X ; \
    lea stack_slots_used, [stack_slots_used + 1]

// Needs to preserve flags
#define POP(X) \
    xchg vm_pc, vm_hp ; \
    pushf ; \
    test stack_slots_used, stack_slots_used ; \
    je stack_underflow ; \
    mov X, qword ptr [vm_sp] ; \
    add vm_sp, STACK_SLOT_SIZE ; \
    dec stack_slots_used ; \
    popf ; \
    xchg vm_pc, vm_hp

// Needs to preserve flags
#define PEEK(X, I) \
    xchg vm_pc, vm_hp ; \
    pushf ; \
    cmp I, stack_slots_used ; \
    jae stack_underflow ; \
    mov X, qword ptr [vm_sp + I * STACK_SLOT_SIZE] ; \
    popf ; \
    xchg vm_pc, vm_hp

#define JMP_IF_NULL(R64, LABEL) \
    cmp R64, TAGGED_NULL ; \
    je LABEL

#define JMP_IF_INT(R64, LABEL) \
    PUSH(R64) ; \
    and R64, INT_MASK ; \
    test R64, R64 ; \
    POP(R64) ; \
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

.section .text

stack_overflow:
    ud2

stack_underflow:
    ud2

zero_location:
    .quad 0

.global interpret
interpret:
    mov vm_hp, rsp   // vm heap pointer
    mov vm_pc, rdi   // vm instruction pointer
    mov vm_sp, rsi   // vm stack pointer
    xor stack_slots_used, stack_slots_used // 0 stack slots are in use
    ret

.section .text.add1
.global add1
add1:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_NOT_INT(rax, 1f)
    UNTAG_INT(rax)
    inc rax
    TAG_INT(rax)
    PUSH(rax)
    ret
1:
    ud2

.section .text.sub1
.global sub1
sub1:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_NOT_INT(rax, 1f)
    UNTAG_INT(rax)
    dec rax
    TAG_INT(rax)
    PUSH(rax)
    ret
1:
    ud2

.section .text.add
.global add
add:
    pop rdi // arity
    test rdi, rdi
    jne 1f
    PUSH(TAG_CONST_INT(0))
    ret
1:
    POP(rax)
    JMP_IF_NOT_INT(rax, 3f)
    UNTAG_INT(rax)
2:
    // base case
    cmp rdi, 1
    jne 1f
    TAG_INT(rax)
    PUSH(rax)
    ret
1:
    POP(rcx)
    JMP_IF_NOT_INT(rcx, 3f)
    UNTAG_INT(rcx)
    add rax, rcx
    dec rdi
    jmp 2b
3:
    ud2

.section .text.sub
.global sub
sub:
    pop rdi // arity
    test rdi, rdi
    jne 1f
    ud2 // can't have (-)
1:
    POP(rax)
    JMP_IF_NOT_INT(rax, 2f)
    UNTAG_INT(rax)
1:
    cmp rdi, 1
    jne 1f
    // unary special case
    neg rax
    TAG_INT(rax)
    PUSH(rax)
    ret
1:
    POP(rcx)
    JMP_IF_NOT_INT(rcx, 2f)
    UNTAG_INT(rcx)
    sub rax, rcx
    dec rdi
    cmp rdi, 1
    jne 1b
    TAG_INT(rax)
    PUSH(rax)
    ret
2:
    ud2

.section .text.mul
.global mul
mul:
    pop rdi // arity
    test rdi, rdi
    jne 1f
    PUSH(TAG_CONST_INT(1))
    ret
1:
    POP(rax)
    JMP_IF_NOT_INT(rax, 3f)
    UNTAG_INT(rax)
2:
    // base case
    cmp rdi, 1
    jne 1f
    TAG_INT(rax)
    PUSH(rax)
    ret
1:
    POP(rcx)
    JMP_IF_NOT_INT(rcx, 3f)
    UNTAG_INT(rcx)
    mul rcx
    dec rdi
    jmp 2b
3:
    ud2


.section .text.lt
.global lt
lt:
    pop rdi // arity
    mov esi, 1 // result
    test rdi, rdi
    jne 1f
    // 0 args
    PUSH(TRUE)
    ret
1:
    POP(rax)
    JMP_IF_NOT_INT(rax, 2f)
    UNTAG_INT(rax)
    dec rdi
    test rdi, rdi
    jne 1f
    // 1 arg
    PUSH(TRUE)
    ret
1:
    POP(rcx)
    JMP_IF_NOT_INT(rcx, 2f)
    UNTAG_INT(rcx)
    cmp rax, rcx
    setl al
    and sil, al
    mov rax, rcx
    dec rdi
    test rdi, rdi
    jne 1b
    TAG_BOOL(rsi)
    PUSH(rsi)
    ret
2:
    ud2

.section .text.eq
.global eq 
eq:
    pop rdi // arity
    mov esi, 1 // result
    test rdi, rdi
    jne 1f
    // 0 args
    PUSH(TRUE)
    ret
1:
    POP(rax)
    JMP_IF_NOT_INT(rax, 2f)
    UNTAG_INT(rax)
    dec rdi
    test rdi, rdi
    jne 1f
    // 1 arg
    PUSH(TRUE)
    ret
1:
    POP(rcx)
    JMP_IF_NOT_INT(rcx, 2f)
    UNTAG_INT(rcx)
    cmp rax, rcx
    sete al
    and sil, al
    mov rax, rcx
    dec rdi
    test rdi, rdi
    jne 1b
    TAG_BOOL(rsi)
    PUSH(rsi)
    ret
2:
    ud2

.section .text.eqp
.global eqp
eqp:
    pop rdi // arity
    mov esi, 1 // result
    test rdi, rdi
    jne 1f
    // 0 args
    PUSH(TRUE)
    ret
1:
    POP(rax)
    dec rdi
    test rdi, rdi
    jne 1f
    // 1 arg
    PUSH(TRUE)
    ret
1:
    POP(rcx)
    cmp rax, rcx
    sete al
    and sil, al
    mov rax, rcx
    dec rdi
    test rdi, rdi
    jne 1b
    TAG_BOOL(rsi)
    PUSH(rsi)
    ret
2:
    ud2

.section .text.load
.global load
load:
    pop rax   // Read immediate into rax
    PUSH(rax) // Push it to interpreter stack
    ret

.section .text.zerop
.global zerop
zerop:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1: // int
    UNTAG_INT(rax)
    test rax, rax
    mov eax, 0 // cannot be xor because of flags
    sete al
    TAG_BOOL(rax)
    PUSH(rax)
    ret

.section .text.integerp
.global integerp
integerp:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_INT(rax, 1f)
    PUSH(FALSE)
    ret
1: // int
    PUSH(TRUE)
    ret

.section .text.booleanp
.global booleanp
booleanp:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_BOOL(rax, 1f)
    PUSH(FALSE)
    ret
1: // bool
    PUSH(TRUE)
    ret

.section .text.charp
.global charp
charp:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_CHAR(rax, 1f)
    PUSH(FALSE)
    ret
1: // char
    PUSH(TRUE)
    ret

.section .text.nullp
.global nullp
nullp:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_NULL(rax, 1f)
    PUSH(FALSE)
    ret
1: // '()
    PUSH(TRUE)
    ret

.section .text.not
.global not
not:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_BOOL(rax, 1f)
    PUSH(FALSE)
    ret
2: // bool
    UNTAG_BOOL(rax)
    not rax
    and rax, 1
    TAG_BOOL(rax)
    PUSH(rax)
    ret

.section .text.chartoint
.global chartoint
chartoint:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_CHAR(rax, 1f)
    ud2
1: // char
    UNTAG_CHAR(rax)
    TAG_INT(rax)
    PUSH(rax)
    ret

.section .text.inttochar
.global inttochar
inttochar:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1: // int
    UNTAG_INT(rax)
    cmp rax, 128
    jb 1f
    ud2
1: // in range
    TAG_CHAR(rax)
    PUSH(rax)
    ret

.section .text.cjump
.global cjump
cjump:
    pop rax
    POP(rcx)
    JMP_IF_BOOL(rcx, 1f)
    ud2
1:
    cmp rcx, FALSE
    je 1f
    shl rax, 4
    lea vm_pc, [vm_pc + rax]
1:
    ret

.section .text.get
.global get
get:
    pop rax // The offset from the stack base to get
    cmp rax, stack_slots_used
    jae 1f
    neg rax
    add rax, stack_slots_used
    mov rax, qword ptr [vm_sp + rax * STACK_SLOT_SIZE - STACK_SLOT_SIZE]
    PUSH(rax)
    ret
1:
    ud2 // Stack access out of range

.section .text.fall
.global fall
fall:
    pop rdi // amount to fall
1:
    test rdi, rdi
    je 1f
    POP(rax)
    POP(rcx) // we use POP for the bounds check. Otherwise could be sub
    PUSH(rax)
    dec rdi
    jmp 1b
1:
    ret

.section .text.forget
.global forget
forget:
    add vm_pc, 8
    POP(rax)
    ret

.section .text.jump
.global jump
jump:
    pop rax
    shl rax, 4
    lea vm_pc, [vm_pc + rax]
    ret

.section .text.cons
.global cons
cons:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax) // cdr
    POP(rcx) // car
    sub vm_hp, 16 // alloc(16)
    mov rdi, vm_hp // heap pointer
    mov qword ptr [rdi], rcx
    mov qword ptr [rdi + 8], rax
    or rdi, 1
    PUSH(rdi)
    ret

.section .text.car
.global car
car:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_PAIR(rax, 1f)
    ud2
1:
    UNTAG_PAIR(rax)
    mov rax, qword ptr [rax]
    PUSH(rax)
    ret

.section .text.cdr
.global cdr
cdr:
    add vm_pc, IMMEDIATE_SIZE
    POP(rax)
    JMP_IF_PAIR(rax, 1f)
    ud2
1:
    UNTAG_PAIR(rax)
    mov rax, qword ptr [rax + 8]
    PUSH(rax)
    ret

.section .text.string
.global string
string:
    pop rdi // arity
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
    TAG_STRING(rax)
    PUSH(rax)
    ret
1:
    POP(rcx)
    JMP_IF_CHAR(rcx, 1f)
    ud2
1:
    UNTAG_CHAR(rcx)
    mov byte ptr [rax + rsi + 8], cl
    inc rsi
    dec rdi
    jmp 2b

.section .text.stringref
.global stringref
stringref:
    add vm_pc, 8
    POP(rcx) // index
    JMP_IF_INT(rcx, 1f)
    ud2
1:
    UNTAG_INT(rcx)
    POP(rax) // string
    JMP_IF_NOT_STRING(rax, 1f)
    UNTAG_STRING(rax)
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f
    mov al, byte ptr [rax + rcx + 8]
    and rax, 0xff
    TAG_CHAR(rax)
    PUSH(rax)
    ret
1:
    ud2

.section .text.stringset
.global stringset
stringset:
    add vm_pc, 8
    POP(rsi) // char
    JMP_IF_CHAR(rsi, 1f)
    ud2
1:
    UNTAG_CHAR(rsi)
    POP(rcx) // index
    JMP_IF_INT(rcx, 1f)
    ud2
1:
    UNTAG_INT(rcx)
    POP(rax) // string
    JMP_IF_NOT_STRING(rax, 1f)
    UNTAG_STRING(rax)
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f // index out of bounds
    mov byte ptr [rax + rcx + 8], sil
    and rax, 0xff
    TAG_CHAR(rax)
    PUSH(rax)
    ret
1:
    ud2

.section .text.stringappend
.global stringappend
stringappend:
    pop rdx // arity
    test rdx, rdx
    jne 1f
    // no args
    sub vm_hp, 8
    mov qword ptr [vm_hp], 0
    mov rax, vm_hp
    TAG_STRING(rax)
    PUSH(rax)
    ret
1:
    // at least one arg
    xor ecx, ecx // counter
    xor eax, eax // length
1:
    cmp rcx, rdx
    je 1f
    PEEK(rsi, rcx) // args[rcx]
    JMP_IF_NOT_STRING(rsi, 2f)
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
    POP(rsi) // arg
    JMP_IF_NOT_STRING(rsi, 2f)
    UNTAG_STRING(rsi)
    mov rcx, qword ptr [rsi] // count
    add rsi, 8
    rep movsb
    dec rdx
    jmp 1b
1:
    mov rax, vm_hp
    TAG_STRING(rax)
    PUSH(rax)
    ret
2:
    ud2

.section .text.vector
.global vector
vector:
    pop rdi // arity
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
    TAG_VECTOR(rax)
    PUSH(rax)
    ret
1:
    POP(rcx)
    mov qword ptr [rax + rsi * 8 + 8], rcx
    inc rsi
    dec rdi
    jmp 2b

.section .text.vectorref
.global vectorref
vectorref:
    add vm_pc, 8
    POP(rcx) // index
    JMP_IF_INT(rcx, 1f)
    ud2
1:
    UNTAG_INT(rcx)
    POP(rax) // vector
    JMP_IF_NOT_VECTOR(rax, 1f)
    UNTAG_VECTOR(rax)
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f
    mov rax, qword ptr [rax + rcx * 8 + 8]
    PUSH(rax)
    ret
1:
    ud2

.section .text.vectorset
.global vectorset
vectorset:
    add vm_pc, 8
    POP(rsi) // arg
    POP(rcx) // index
    JMP_IF_INT(rcx, 1f)
    ud2
1:
    UNTAG_INT(rcx)
    POP(rax) // vector
    JMP_IF_NOT_VECTOR(rax, 1f)
    UNTAG_VECTOR(rax)
    mov rdi, qword ptr [rax]
    cmp rcx, rax
    jae 1f // index out of bounds
    mov qword ptr [rax + rcx * 8 + 8], rsi
    PUSH(rax)
    ret
1:
    ud2

.section .text.vectorappend
.global vectorappend
vectorappend:
    pop rdx // arity
    test rdx, rdx
    jne 1f
    // no args
    sub vm_hp, 8
    mov qword ptr [vm_hp], 0
    mov rax, vm_hp
    TAG_VECTOR(rax)
    PUSH(rax)
    ret
1:
    // at least one arg
    xor ecx, ecx // counter
    xor eax, eax // length
1:
    cmp rcx, rdx
    je 1f
    PEEK(rsi, rcx) // args[rcx]
    JMP_IF_NOT_VECTOR(rsi, 2f)
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
    POP(rsi) // arg
    JMP_IF_NOT_VECTOR(rsi, 2f)
    UNTAG_VECTOR(rsi)
    mov rcx, qword ptr [rsi] // count
    add rsi, 8
    rep movsq
    dec rdx
    jmp 1b
1:
    mov rax, vm_hp
    TAG_VECTOR(rax)
    PUSH(rax)
    ret
2:
    ud2


.section .text.done
.global done
done:
    test stack_slots_used, stack_slots_used
    jne 1f
    PUSH(UNSPECIFIED)
1:
    cmp stack_slots_used, 1
    jne 1f
    mov rdi, qword ptr [vm_sp]  // Grab what's on top of the stack
    xchg vm_pc, vm_hp             // Put the stack back
    and vm_pc, -16              // Align the stack for a real function call
    call print_value_and_exit // Call back into C
1:
    ud2 // more than one thing left on the stack
