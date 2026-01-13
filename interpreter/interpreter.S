.intel_syntax noprefix

#include "constants.h"

// rbx is our sp
// rsp is our pc
// rbp holds our original rsp, so we can do tricks :)
// r12 holds how many stack slots are currently in use.
// Each instruction is a 64-bit opcode and a maybe-unused 64-bit immediate.
// Each opcode is simply the address of its implementation.
// (See the linker script for these addresses)
// Each immediate is tagged with its type in its low bits as follows:
// -       00 => int
// - 00001111 => char
// -  0011111 => bool
// - 00101111 => '()
// - all 1s   => #<unspecified>

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
    cmp r12, 0 ; \
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

#define TAG_INT(R64) \
    shl R64, 2

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
    xor r12d, r12d // 0 stack slots are in use
    ret

.section .text.add1
.global add1
add1:
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1:
    UNTAG_INT(rax)
    inc rax
    TAG_INT(rax)
    PUSH(rax)
    ret

.section .text.sub1
.global sub1
sub1:
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1:
    UNTAG_INT(rax)
    dec rax
    TAG_INT(rax)
    PUSH(rax)
    ret

.section .text.add
.global add
add:
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1:
    POP(rcx)
    JMP_IF_INT(rcx, 1f)
    ud2
1:
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    add rax, rcx
    TAG_INT(rax)
    PUSH(rax)
    ret

.section .text.sub
.global sub
sub:
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1:
    POP(rcx)
    JMP_IF_INT(rcx, 1f)
    ud2
1:
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    sub rax, rcx
    TAG_INT(rax)
    PUSH(rax)
    ret

.section .text.mul
.global mul
mul:
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1: // int
    POP(rcx)
    JMP_IF_INT(rcx, 1f)
    ud2
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    mul rcx
    TAG_INT(rax)
    PUSH(rax)
    ret

.section .text.lt
.global lt
lt:
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1: // int
    POP(rcx)
    JMP_IF_INT(rcx, 1f)
    ud2
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    cmp rax, rcx
    mov eax, 0 // cannot be xor because of flags
    setl al
    TAG_BOOL(rax)
    PUSH(rax)
    ret

.section .text.eq
.global eq 
eq:
    add rsp, 8 // Skip immediate
    POP(rax)
    JMP_IF_INT(rax, 1f)
    ud2
1: // int
    POP(rcx)
    JMP_IF_INT(rcx, 1f)
    ud2
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    cmp rax, rcx
    mov eax, 0 // cannot be xor because of flags
    sete al
    TAG_BOOL(rax)
    PUSH(rax)
    ret

.section .text.eqp
.global eqp
eqp:
    add rsp, 8 // Skip immediate
    POP(rax)
    POP(rcx)
    cmp rax, rcx
    je 1f
    mov rax, FALSE
    PUSH(rax)
    ret
1:
    mov rax, TRUE
    PUSH(rax)
    ret

.section .text.load64
.global load64
load64:
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
    add rsp, 8 // Skip immediate
    POP(rax)
    POP(rcx)
    PUSH(rax)
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

.section .text.done
.global done
done:
    mov rdi, qword ptr [rbx]  // Grab what's on top of the stack
    mov rsp, rbp              // Put the stack back :)
    and rsp, -16              // Align the stack for a real function call
    call print_value_and_exit // Call back into C
