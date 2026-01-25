typedef unsigned long long uint64_t;
_Static_assert(sizeof(uint64_t) == 8, "uint64_t is not 8 bytes");

typedef long long int64_t;
_Static_assert(sizeof(int64_t) == 8, "int64_t is not 8 bytes");
#define INT64_MIN (int64_t)(-9223372036854775807ll - 1ll)

typedef typeof(sizeof(char)) size_t;

typedef unsigned char uint8_t;
_Static_assert(sizeof(uint8_t) == 1, "uint8_t is not 1 byte");
_Static_assert((uint8_t)-1 == 255, "(uint8_t)-1 != 255");
#define CHAR_BIT 8

typedef long long ssize_t;
typedef long long off_t;

#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1

#define STDIN_FILENO 0
#define STDOUT_FILENO 1

#define PROT_READ 1
#define PROT_WRITE 2

#define MAP_ANONYMOUS 0x20
#define MAP_PRIVATE 0x2
#define MAP_FIXED_NOREPLACE 0x100000
#define MAP_GROWSDOWN 0x100

#define NULL ((void *)0)

[[noreturn]] void exit(int);

void *mmap(void *addr, size_t len, int prot, int flags, int fd, off_t off);

int mprotect(void *addr, size_t len, int prot);

ssize_t read(int fd, void *buf, size_t count);

ssize_t write(int fd, void const *buf, size_t count);
