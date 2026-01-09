.intel_syntax noprefix

#include "constants.h"

// rbx is our sp
// rsp is our pc
// rbp will hold our original rsp, so we can do tricks :)
// Each instruction is a 64-bit opcode and a maybe-unused 64-bit immediate.
// Each opcode is simply the address of its implementation.
// (See the linker script for these addresses)
// Each immediate is tagged with its type in its low bits as follows:
// -       00 => int
// - 00001111 => char
// -  0011111 => bool
// - 00101111 => '()


// Needs to preserve flags
#define PUSH(X) \
    lea rbx, [rbx + 8] ; \
    mov qword ptr [rbx], X

// Needs to preserve flags
#define POP(X) \
    mov X, qword ptr [rbx] ; \
    lea rbx, [rbx - 8]

#define TYPE_DISPATCH(VAL, INT_LABEL, BOOL_LABEL, CHAR_LABEL, NULL_LABEL) \
    cmp VAL, TAGGED_NULL ; \
    je NULL_LABEL ; \
    cmp VAL, TRUE ; \
    je BOOL_LABEL ; \
    cmp VAL, FALSE ; \
    je BOOL_LABEL ; \
    PUSH(VAL) ; \
    and VAL, INT_MASK ; \
    test VAL, VAL ; \
    POP(VAL) ; \
    je INT_LABEL ; \
    PUSH(VAL) ; \
    sub VAL, TAGGED_CHAR_MIN ; \
    cmp VAL, TAGGED_CHAR_MAX ; \
    POP(VAL) ; \
    jbe CHAR_LABEL ; \
    ud2

#define TAG_INT(VAL) \
    shl VAL, 2

#define UNTAG_INT(VAL) \
    shr VAL, 2

#define TAG_BOOL(VAL) \
    shl VAL, 7 ; \
    or VAL, BOOL_SUFFIX

#define UNTAG_BOOL(VAL) \
    shr VAL, 7

#define UNTAG_CHAR(VAL) \
    shr VAL, 8

#define TAG_CHAR(VAL) \
    shl VAL, 8 ; \
    or VAL, CHAR_SUFFIX

.section .text

.global interpret
interpret:
    mov rbp, rsp
    mov rsp, rdi // Load up the interpreter pc
    mov rbx, rsi // Load up the interpreter sp
    ret

.section .text.add1
.global add1
add1:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    inc rax
    TAG_INT(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.sub1
.global sub1
sub1:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    dec rax
    TAG_INT(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.add
.global add
add:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    POP(rcx)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    add rax, rcx
    TAG_INT(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.sub
.global sub
sub:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    POP(rcx)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    sub rax, rcx
    TAG_INT(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.mul
.global mul
mul:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    POP(rcx)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    mul rcx
    TAG_INT(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.lt
.global lt
lt:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    POP(rcx)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    cmp rax, rcx
    mov eax, 0
    setl al
    TAG_BOOL(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.eq
.global eq 
eq:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    POP(rcx)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    UNTAG_INT(rcx)
    cmp rax, rcx
    mov eax, 0
    sete al
    TAG_BOOL(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.load64
.global load64
load64:
    pop rax   // Read immediate into rax
    PUSH(rax) // Push it to interpreter stack
    ret

.section .text.zeroq
.global zeroq
zeroq:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    UNTAG_INT(rax)
    test rax, rax
    mov eax, 0
    sete al
    TAG_BOOL(rax)
    PUSH(rax)
    ret
2: // not int
    ud2

.section .text.integerq
.global integerq
integerq:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 2f, 2f)
1: // int
    PUSH(TRUE)
    ret
2: // not int
    PUSH(FALSE)
    ret

.section .text.booleanq
.global booleanq
booleanq:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 2f, 1f, 2f, 2f)
1: // bool
    PUSH(TRUE)
    ret
2: // not bool
    PUSH(FALSE)
    ret

.section .text.charq
.global charq
charq:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 2f, 2f, 1f, 2f)
1: // char
    PUSH(TRUE)
    ret
2: // not char
    PUSH(FALSE)
    ret

.section .text.nullq
.global nullq
nullq:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 2f, 2f, 2f, 1f)
1: // '()
    PUSH(TRUE)
    ret
2: // not '()
    PUSH(FALSE)
    ret

.section .text.not
.global not
not:
    add rsp, 8 // Skip immediate
    POP(rax)
    TYPE_DISPATCH(rax, 1f, 2f, 1f, 1f)
1: // not bool
    PUSH(FALSE)
    ret
2: // bool
    UNTAG_BOOL(rax)
    not rax
    and rax, 1
    TAG_BOOL(rax)
    PUSH(rax)
    ret

.section .text.done
.global done
done:
    mov rdi, qword ptr [rbx]  // Grab what's on top of the stack
    mov rsp, rbp              // Put the stack back :)
    and rsp, -16              // Align the stack for a real function call
    call print_value_and_exit // Call back into C
