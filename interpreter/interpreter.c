#define _GNU_SOURCE

#include <stdlib.h> // for EXIT_*, exit
#include <stdio.h> // for getdelim, stdin, EOF, printf
#include <sys/mman.h> // for mmap, PROT_*, MAP_*
#include <stdint.h> // for uint64_t
#include <inttypes.h> // for PRIu64
#include <signal.h> // for sigset_t, sigfillset, sigprocmask, SIG_SETMASK

#include "constants.h"

void interpret(void *ip, void *sp);

size_t const STACK_SIZE = 0x10000;

int main(void) {
    // Block all signals because they clobber the red zone
    sigset_t mask;
    sigfillset(&mask);
    sigprocmask(SIG_SETMASK, &mask, NULL);

    char * bytecode = nullptr;
    size_t bytecode_size;
    ssize_t bytes_read = getdelim(&bytecode, &bytecode_size, EOF, stdin);
    if (bytes_read < 0) {
        return EXIT_FAILURE;
    }
    if (bytes_read % 8) {
        return EXIT_FAILURE;
    }

    void * const stack = mmap(NULL, STACK_SIZE, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
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
    } else {
        printf("Exit value is malformed: %" PRIu64 "\n", v);
        exit(EXIT_FAILURE);
    }
    exit(EXIT_SUCCESS);
}
