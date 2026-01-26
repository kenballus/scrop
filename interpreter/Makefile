.PHONY: all clean fmt

TARGET := interpreter

all: $(TARGET)

CC := clang
CFLAGS := -O0 -std=c23 -Wall -Wextra -Wpedantic -Wvla -Wshadow -Wno-c2y-extensions -fno-stack-protector -MMD -MP
LDFLAGS := -T layout.ld -static -nostdlib

SRCS := $(wildcard *.c) $(wildcard *.S)
OBJS := $(addsuffix .o, $(basename $(SRCS)))
DEPS := $(OBJS:.o=.d)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

%.o: %.S
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) $^ -o $@

fmt:
	clang-format --style='{IndentWidth: 4}' -i *.c *.h

-include $(DEPS)

clean:
	rm -f $(TARGET) $(OBJS) $(DEPS)
