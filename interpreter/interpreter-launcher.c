#define _GNU_SOURCE

#include "constants.h"
#include "libc.h"
#include "interpreter.h"

static void *mmap_or_die(void *addr, size_t len, int prot, int flags, int fd,
                  off_t off) {
    void *result = mmap(addr, len, prot, flags, fd, off);
    if (result <= NULL) {
        exit(EXIT_FAILURE);
    }
    return result;
}

void _start(void) {
    size_t capacity = PAGESIZE;
    unsigned char *bytecode = mmap_or_die(NULL, capacity, PROT_READ | PROT_WRITE,
                                    MAP_ANONYMOUS | MAP_PRIVATE, -1, 0);
    size_t bytes_read = 0;
    while (true) {
        ssize_t const read_rc =
            read(STDIN_FILENO, bytecode + bytes_read, capacity - bytes_read);
        if (read_rc < 0) {
            exit(EXIT_FAILURE);
        }
        if (read_rc == 0) {
            break;
        }
        bytes_read += read_rc;
        if (bytes_read == capacity) {
            mmap_or_die(bytecode + capacity, PAGESIZE, PROT_READ | PROT_WRITE,
                        MAP_ANONYMOUS | MAP_PRIVATE | MAP_FIXED_NOREPLACE, -1,
                        0);
            capacity += PAGESIZE;
        }
    }

    if (mprotect(bytecode, capacity, PROT_READ) != 0) {
        exit(EXIT_FAILURE);
    }

    uint8_t *stack =
        mmap_or_die(NULL, PAGESIZE, PROT_READ | PROT_WRITE,
                    MAP_ANONYMOUS | MAP_PRIVATE | MAP_GROWSDOWN, -1, 0);
    interpret(bytecode, stack + PAGESIZE);
}

static void write_or_die(int const fd, void const *const buf, size_t const len) {
    ssize_t const write_rc = write(fd, buf, len);
    if (write_rc < 0 || (write_rc == 0 && len > 0)) {
        exit(EXIT_FAILURE);
    }
}

#define PRINT_STRING_LITERAL(S) write_or_die(STDOUT_FILENO, S, sizeof(S) - 1)

static void print_char_or_die(char const c) {
    write_or_die(STDOUT_FILENO, &c, sizeof(c));
}

static void print_i64_or_die(int64_t v) {
    if (v == INT64_MIN) {
        // Not safe to negate INT64_MIN, so handle specially
        PRINT_STRING_LITERAL("-9223372036854775808");
        return;
    }
    if (v == 0) {
        // The algorithm below only works for nonzero values
        PRINT_STRING_LITERAL("0");
        return;
    }

    static char result[21 /* len(str(-(2**63 - 1))) */];

    size_t bytes_written = 0;
    if (v < 0) {
        result[bytes_written] = '-';
        bytes_written++;
        v *= -1;
    }
    uint64_t const uv = v;
    uint64_t power_of_ten = 10000000000000000000ull; // (smallest power of ten greater than INT64_MAX)
    while (power_of_ten > uv) {
        power_of_ten /= 10;
    }
    power_of_ten *= 10;
    while (power_of_ten > 1) {
        result[bytes_written] = '0' + ((uv % power_of_ten) / (power_of_ten / 10));
        power_of_ten /= 10;
        bytes_written++;
    }
    write_or_die(STDOUT_FILENO, result, bytes_written);
}

static void print_value(uint64_t const v) {
    if ((v & INT_MASK) == INT_SUFFIX) {
        int64_t untagged_v = v >> 2;
        if (untagged_v >= 0x2000000000000000ll) {
            untagged_v += -0x4000000000000000ll;
        }
        print_i64_or_die(untagged_v);
    } else if (v == TRUE) {
        PRINT_STRING_LITERAL("#t");
    } else if (v == FALSE) {
        PRINT_STRING_LITERAL("#f");
    } else if ((v & CHAR_MASK) == CHAR_SUFFIX) {
        PRINT_STRING_LITERAL("#\\");
        print_char_or_die((char)(v >> 8));
    } else if (v == TAGGED_NULL) {
        PRINT_STRING_LITERAL("'()");
    } else if ((v & PAIR_MASK) == PAIR_SUFFIX) {
        uint64_t car = *(uint64_t *)(v - 1);
        uint64_t cdr = *(uint64_t *)((v - 1) + 8);
        PRINT_STRING_LITERAL("(");
        print_value(car);
        PRINT_STRING_LITERAL(" . ");
        print_value(cdr);
        PRINT_STRING_LITERAL(")");
    } else if ((v & STRING_MASK) == STRING_SUFFIX) {
        uint64_t const len = *(uint64_t *)(v - 3);
        PRINT_STRING_LITERAL("\"");
        write_or_die(STDOUT_FILENO, (void *)(v + 5), len);
        PRINT_STRING_LITERAL("\"");
    } else if ((v & VECTOR_MASK) == VECTOR_SUFFIX) {
        uint64_t const len = *(uint64_t *)(v - 2);
        PRINT_STRING_LITERAL("#(");
        for (uint64_t i = 0; i < len; i++) {
            print_value(((uint64_t *)(v - 2 + 8))[i]);
            if (i != len - 1) {
                PRINT_STRING_LITERAL(" ");
            }
        }
        PRINT_STRING_LITERAL(")");
    } else if (v != UNSPECIFIED) {
        PRINT_STRING_LITERAL("value is malformed.\n");
        exit(EXIT_FAILURE);
    }
}

void print_value_and_exit(uint64_t const v) {
    print_value(v);
    PRINT_STRING_LITERAL("\n");
    exit(EXIT_SUCCESS);
}
