.PHONY: all clean fmt

all: interpreter

CC := clang
CFLAGS := -O0 -std=c23 -Wall -Wextra -Wpedantic -Wvla -Wshadow -Wno-c2y-extensions
LDFLAGS := -T layout.ld

interpreter: interpreter.c interpreter.S
	$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@

fmt:
	clang-format --style='{IndentWidth: 4}' -i interpreter.c

clean:
	rm -f interpreter
