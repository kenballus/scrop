.PHONY: all clean fmt

all: interpreter

CC := clang
CFLAGS := -O0 -std=c23 -Wall -Wextra -Wpedantic -Wvla -Wshadow -Wno-c2y-extensions -nostdlib -fno-stack-protector
LDFLAGS := -T layout.ld -static

interpreter-launcher.o: interpreter-launcher.c
	$(CC) $(CFLAGS) -c $^ -o $@

interpreter.o: interpreter.S
	$(CC) $(CFLAGS) -c $^ -o $@

libc.o: libc.S
	$(CC) $(CFLAGS) -c $^ -o $@

interpreter: interpreter-launcher.o interpreter.o libc.o
	$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@

fmt:
	clang-format --style='{IndentWidth: 4}' -i *.c *.h

clean:
	rm -f interpreter *.o
