.intel_syntax noprefix

#include "constants.h"

// rbx is our sp
// rsp is our pc
// rbp holds our original rsp, so we can do tricks :)
// rdx is out heap pointer
// r12 holds how many stack slots are currently in use.
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

// Needs to preserve flags
#define PUSH(X) \
    xchg rsp, rbp ; \
    pushf ; \
    cmp r12, NUM_STACK_SLOTS ; \
    je stack_overflow ; \
    add rbx, 8 ; \
    mov qword ptr [rbx], X ; \
    inc r12 ; \
    popf ; \
    xchg rsp, rbp

// Needs to preserve flags
#define POP(X) \
    xchg rsp, rbp ; \
    pushf ; \
    test r12, r12 ; \
    je stack_underflow ; \
    mov X, qword ptr [rbx] ; \
    sub rbx, 8 ; \
    dec r12 ; \
    popf ; \
    xchg rsp, rbp

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

#define TAG_INT(R64) \
    shl R64, 2

#define TAG_CONST_INT(V) \
    V << 2

#define UNTAG_INT(R64) \
    shr R64, 2

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

.section .text

stack_overflow:
    ud2

stack_underflow:
    ud2

zero_location:
    .quad 0

.global interpret
interpret:
    mov rbp, rsp // Save the original rsp
    mov rsp, rdi // Load up the interpreter pc
    lea rbx, [rsi - 8] // Load up the interpreter sp
    // rdx contains heap pointer
    xor r12d, r12d // 0 stack slots are in use
    ret

.section .text.add1
.global add1
add1:
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
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
    lea rsp, [rsp + rax]
1:
    ret

.section .text.get
.global get
get:
    pop rax // The offset from the stack base to get
    cmp rax, r12
    jb 1f
    ud2 // Stack access out of range
1:
    // movq rax, [stack_base + (offset - r12 + 1) * slot_size]
    sub rax, r12
    inc rax
    mov rax, qword ptr [rbx + rax * STACK_SLOT_SIZE]
    PUSH(rax)
    ret

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
    add rsp, 8
    POP(rax)
    ret

.section .text.jump
.global jump
jump:
    pop rax
    shl rax, 4
    lea rsp, [rsp + rax]
    ret

.section .text.cons
.global cons
cons:
    add rsp, 8 // Skip immediate
    POP(rax) // cdr
    POP(rcx) // car
    mov rdi, rdx // heap pointer
    add rdx, 16
    mov qword ptr [rdi], rcx
    mov qword ptr [rdi + 8], rax
    or rdi, 1
    PUSH(rdi)
    ret

.section .text.car
.global car
car:
    add rsp, 8 // Skip immediate
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
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_PAIR(rax, 1f)
    ud2
1:
    UNTAG_PAIR(rax)
    mov rax, qword ptr [rax + 8]
    PUSH(rax)
    ret

.section .text.done
.global done
done:
    test r12, r12
    jne 1f
    PUSH(UNSPECIFIED)
1:
    cmp r12, 1
    jne 1f
    mov rdi, qword ptr [rbx]  // Grab what's on top of the stack
    mov rsp, rbp              // Put the stack back :)
    and rsp, -16              // Align the stack for a real function call
    call print_value_and_exit // Call back into C
1:
    ud2 // more than one thing left on the stack
