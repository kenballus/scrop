import sys
import os
from dataclasses import dataclass


@dataclass
class Char:
    value: str

    def __post_init__(self):
        assert len(self.value) == 1


class Unspecified:
    pass


Immediate = int | bool | Char | None | Unspecified

CHAR_PREFIX: str = "#\\"


def parse_immediate(s: str) -> Immediate:
    match s:
        case "#f" | "#F":
            return False
        case "#t" | "#T":
            return True
        case "NULL":
            return None
        case "UNSPECIFIED":
            return Unspecified()

    if s.isnumeric() and s.isascii():
        result: int = int(s)
        if result < 0 or result > 2**62 - 1:
            raise ValueError("Integer {result} out of range")
        return result

    if s.startswith(CHAR_PREFIX):
        char_data: str = s[len(CHAR_PREFIX) :]
        if len(char_data) == 1 and char_data.isascii():
            return Char(char_data)
        elif len(char_data) == 3 and char_data[0] == "x" and char_data.isascii():
            return Char(chr(int(char_data[1:], 16)))
        raise ValueError(
            f"Couldn't parse character constant {s} (not all are supported yet)"
        )

    raise ValueError(f"Couldn't parse immediate {s}")


def serialize_immediate(v: Immediate) -> bytes:
    if isinstance(v, bool):
        return (0b10011111 if v else 0b00011111).to_bytes(8, "little")
    if isinstance(v, int):
        return (v << 2).to_bytes(8, "little")
    if isinstance(v, Char):
        return ((ord(v.value) << 8) | 0b00001111).to_bytes(8, "little")
    if v is None:
        return 0b00101111.to_bytes(8, "little")
    if isinstance(v, Unspecified):
        return 0xFFFFFFFFFFFFFFFF.to_bytes(8, "little")
    assert False


DEFAULT_IMMEDIATE: bytes = 0x0.to_bytes(8, "little")


def main() -> None:
    for line in filter(lambda l: l.strip(), sys.stdin.readlines()):
        opcode: int | None = None
        immediate: bytes = DEFAULT_IMMEDIATE
        match line.split():
            case ["LOAD", v]:
                opcode = 0x10AD000
                immediate = serialize_immediate(parse_immediate(v))
            case ["JUMP", v]:
                opcode = 0x70AD000
                immediate = int(v).to_bytes(8, "little")
            case ["CJUMP", v]:
                opcode = 0xCA7000
                immediate = int(v).to_bytes(8, "little")
            case ["GET", v]:
                opcode = 0x9E7000
                immediate = int(v).to_bytes(8, "little")
            case ["FORGET"]:
                opcode = 0x49E7000
            case ["ADD1"]:
                opcode = 0xADD1000
            case ["SUB1"]:
                opcode = 0x50B1000
            case ["ADD", v]:
                opcode = 0x0ADD000
                immediate = int(v).to_bytes(8, "little")
            case ["SUB", v]:
                opcode = 0x050B000
                immediate = int(v).to_bytes(8, "little")
            case ["MUL", v]:
                opcode = 0x0A55000
                immediate = int(v).to_bytes(8, "little")
            case ["LT", v]:
                opcode = 0x1700000
                immediate = int(v).to_bytes(8, "little")
            case ["EQ", v]:
                opcode = 0xE3E3000
                immediate = int(v).to_bytes(8, "little")
            case ["EQP", v]:
                opcode = 0x3E3E000
                immediate = int(v).to_bytes(8, "little")
            case ["STRING", v]:
                opcode = 0x571f000
                immediate = int(v).to_bytes(8, "little")
            case ["ZEROP"]:
                opcode = 0xEEEE000
            case ["STRINGREF"]:
                opcode = 0x571e000
            case ["STRINGSET"]:
                opcode = 0x5715000
            case ["STRINGAPPEND", v]:
                opcode = 0x571A000
                immediate = int(v).to_bytes(8, "little")
            case ["INTEGERP"]:
                opcode = 0x1234000
            case ["BOOLEANP"]:
                opcode = 0xB001000
            case ["CHARP"]:
                opcode = 0xCACA000
            case ["NULLP"]:
                opcode = 0x4321000
            case ["NOT"]:
                opcode = 0x7777000
            case ["INTTOCHAR"]:
                opcode = 0x170C000
            case ["CHARTOINT"]:
                opcode = 0xC701000
            case ["FALL", v]:
                opcode = 0xFA11000
                immediate = int(v).to_bytes(8, "little")
            case ["CONS"]:
                opcode = 0xC0C0000
            case ["CAR"]:
                opcode = 0xCA00000
            case ["CDR"]:
                opcode = 0xCD00000
            case _:
                raise ValueError(f"Couldn't parse line {line}")
        os.write(1, opcode.to_bytes(8, "little") + immediate)
    os.write(1, 0xD0D0000.to_bytes(8, "little") + DEFAULT_IMMEDIATE)


if __name__ == "__main__":
    main()
