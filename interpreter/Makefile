.PHONY: all clean fmt

all: interpreter

CC := clang
CFLAGS := -O0 -std=c23
LDFLAGS := -T layout.ld

interpreter: interpreter.c interpreter.S
	$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@

fmt:
	clang-format --style='{IndentWidth: 4}' -i interpreter.c

clean:
	rm -f interpreter
