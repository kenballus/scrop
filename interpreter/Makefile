.PHONY: all clean

all: interpreter

CC := clang
CFLAGS := -O2 -std=c23
LDFLAGS := -T layout.ld

interpreter: interpreter.c interpret.S
	$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@

clean:
	rm -f interpreter
