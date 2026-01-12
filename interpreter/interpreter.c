#define _GNU_SOURCE

#include <inttypes.h> // for PRIu64
#include <signal.h>   // for sigset_t, sigfillset, sigprocmask, SIG_SETMASK
#include <stddef.h>   // for NULL
#include <stdint.h>   // for uint64_t
#include <stdio.h>    // for getdelim, stdin, EOF, printf, stderr, fputs
#include <stdlib.h>   // for EXIT_*, exit
#include <sys/mman.h> // for mmap, PROT_*, MAP_*

#include "constants.h"

[[noreturn]] void interpret(void *ip, void *sp);

bool is_valid_opcode(uint64_t const opcode) {
    static uint64_t const OPCODES[] = {
        0xadd1000, 0x50b1000, 0xd0d0000, 0x10ad000, 0x0add000, 0x050b000,
        0x0a55000, 0x1001000, 0xe3e3000, 0xeeee000, 0x1234000, 0xb001000,
        0x70ad000, 0x4321000, 0x7777000, 0xcaca000, 0xc701000, 0x170c000};
    for (size_t i = 0; i < _Countof(OPCODES); i++) {
        if (opcode == OPCODES[i]) {
            return true;
        }
    }
    return false;
}

bool is_valid_bytecode(uint64_t const *const bytecode,
                       ssize_t const bytecode_size) {
    if (bytecode_size < 0 || bytecode_size % 16) {
        return false;
    }
    ssize_t num_bytecode_words = bytecode_size / 16;
    for (ssize_t i = 0; i < num_bytecode_words; i++) {
        if (!is_valid_opcode(bytecode[i * 2])) {
            return false;
        }
    }
    return true;
}

int main(void) {
    // Block all signals because they clobber the red zone
    sigset_t mask;
    sigfillset(&mask);
    sigprocmask(SIG_SETMASK, &mask, NULL);

    uint64_t *bytecode = NULL;
    size_t bytecode_allocation_size;
    ssize_t bytes_read =
        getdelim((char **)&bytecode, &bytecode_allocation_size, EOF, stdin);
    if (!is_valid_bytecode(bytecode, bytes_read)) {
        fputs("Invalid bytecode.\n", stderr);
        return EXIT_FAILURE;
    }

    void *const stack = mmap(NULL, STACK_SIZE_IN_BYTES, PROT_READ | PROT_WRITE,
                             MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (!stack) {
        return EXIT_FAILURE;
    }

    interpret(bytecode, stack);
}

void print_value_and_exit(uint64_t v) {
    if ((v & INT_MASK) == INT_SUFFIX) {
        printf("%" PRIu64 "\n", v >> 2);
    } else if (v == TRUE) {
        puts("#t");
    } else if (v == FALSE) {
        puts("#f");
    } else if ((v & CHAR_MASK) == CHAR_SUFFIX) {
        printf("#\\%c\n", (char)(v >> 8));
    } else if (v == TAGGED_NULL) {
        puts("'()");
    } else {
        printf("Exit value is malformed: %" PRIu64 "\n", v);
        exit(EXIT_FAILURE);
    }
    exit(EXIT_SUCCESS);
}
