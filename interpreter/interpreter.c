#define _GNU_SOURCE

#include <inttypes.h> // for PRIu64, PRIx64
#include <signal.h>   // for sigset_t, sigfillset, sigprocmask, SIG_SETMASK
#include <stddef.h>   // for NULL
#include <stdint.h>   // for uint64_t, intmax_t
#include <stdio.h>    // for getdelim, stdin, EOF, printf, stderr, fputs
#include <stdlib.h>   // for EXIT_*, exit
#include <sys/mman.h> // for mmap, PROT_*, MAP_*

#include "constants.h"

[[noreturn]] void interpret(void *ip, void *sp);

bool is_valid_opcode(uint64_t const opcode) {
    static uint64_t const OPCODES[] = {
        0xadd1000, 0x50b1000, 0xd0d0000, 0x10ad000, 0x0add000, 0x050b000,
        0x0a55000, 0x1001000, 0xe3e3000, 0xeeee000, 0x1234000, 0xb001000,
        0xca7000, 0x70ad000, 0x4321000, 0x7777000, 0xcaca000, 0xc701000, 0x170c000, 0x3e3e000, 0x9e7000, 0x49e7000, 0xfa11000};
    for (size_t i = 0; i < _Countof(OPCODES); i++) {
        if (opcode == OPCODES[i]) {
            return true;
        }
    }
    return false;
}

void validate_bytecode(uint64_t const *const bytecode,
                       size_t const bytecode_size) {
    if (bytecode_size < 0 || bytecode_size % 16) {
        fprintf(stderr, "Invalid bytecode size %jd\n", (intmax_t)bytecode_size);
        exit(EXIT_FAILURE);
    }
    size_t num_bytecode_words = bytecode_size / 16;
    for (size_t i = 0; i < num_bytecode_words; i++) {
        if (!is_valid_opcode(bytecode[i * 2])) {
            fprintf(stderr, "Invalid opcode %" PRIx64 "\n", bytecode[i * 2]);
        }
    }
}

int main(void) {
    // Block all signals because they clobber the red zone
    // TODO: Use sigaltstack
    sigset_t mask;
    sigfillset(&mask);
    sigprocmask(SIG_SETMASK, &mask, NULL);

    size_t const INSTRUCTIONS_PER_READ = 64;
    uint64_t *bytecode = NULL;
    size_t bytes_read = 0;
    while (true) {
        bytecode = realloc(bytecode, bytes_read + INSTRUCTIONS_PER_READ * INSTRUCTION_SIZE);
        if (bytecode == NULL) {
            return EXIT_FAILURE;
        }
        size_t const fread_rc = fread(bytecode, INSTRUCTION_SIZE, INSTRUCTIONS_PER_READ, stdin);
        bytes_read += fread_rc * INSTRUCTION_SIZE;
        if (fread_rc != INSTRUCTIONS_PER_READ) {
            break;
        }
    }
    validate_bytecode(bytecode, bytes_read);

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
    } else if (v != UNSPECIFIED) {
        printf("Exit value is malformed: %" PRIu64 "\n", v);
        exit(EXIT_FAILURE);
    }
    exit(EXIT_SUCCESS);
}
