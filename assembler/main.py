import sys
import os
from dataclasses import dataclass

@dataclass
class Char:
    value: str
    def __post_init__(self):
        assert(len(self.value) == 1)

class Unspecified:
    pass

Value = int | bool | Char | None | Unspecified

CHAR_PREFIX: str = "#\\"

def parse_immediate(s: str) -> Value:
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
        char_data: str = s[len(CHAR_PREFIX):]
        if len(char_data) == 1 and char_data.isascii():
            return Char(char_data)
        elif len(char_data) == 3 and char_data[0] == "x" and char_data.isascii():
            return Char(chr(int(char_data[1:], 16)))
        raise ValueError(f"Couldn't parse character constant {s} (not all are supported yet)")

    raise ValueError(f"Couldn't parse immediate {s}")


def serialize_immediate(v: Value) -> bytes:
    if isinstance(v, bool):
        return (0b10011111 if v else 0b00011111).to_bytes(8, "little")
    if isinstance(v, int):
        return (v << 2).to_bytes(8, "little")
    if isinstance(v, Char):
        return ((ord(v.value) << 8) | 0b00001111).to_bytes(8, "little")
    if v is None:
        return 0b00101111.to_bytes(8, "little")
    if isinstance(v, Unspecified):
        return 0xffffffffffffffff.to_bytes(8, "little")
    assert False


DEFAULT_IMMEDIATE: bytes = 0x0.to_bytes(8, "little")
def main() -> None:
    for line in filter(lambda l: l.strip(), sys.stdin.readlines()):
        opcode: int | None = None
        immediate: bytes = DEFAULT_IMMEDIATE
        match line.split():
            case ["LOAD64", v]:
                opcode = 0x10AD000
                immediate = serialize_immediate(parse_immediate(v))
            case ["JUMP", v]:
                opcode = 0x70ad000
                immediate = int(v).to_bytes(8, "little")
            case ["CJUMP", v]:
                opcode = 0xca7000
                immediate = int(v).to_bytes(8, "little")
            case ["GET", v]:
                opcode = 0x9e7000
                immediate = int(v).to_bytes(8, "little")
            case ["FORGET"]:
                opcode = 0x49e7000
            case ["EQP"]:
                opcode = 0x3e3e000
            case ["ADD1"]:
                opcode = 0xADD1000
            case ["SUB1"]:
                opcode = 0x50B1000
            case ["ADD"]:
                opcode = 0x0ADD000
            case ["SUB"]:
                opcode = 0x050B000
            case ["MUL"]:
                opcode = 0x0a55000
            case ["LT"]:
                opcode = 0x1001000
            case ["EQ"]:
                opcode = 0xe3e3000
            case ["ZEROP"]:
                opcode = 0xEEEE000
            case ["INTEGERP"]:
                opcode = 0x1234000
            case ["BOOLEANP"]:
                opcode = 0xb001000
            case ["CHARP"]:
                opcode = 0xcaca000
            case ["NULLP"]:
                opcode = 0x4321000
            case ["NOT"]:
                opcode = 0x7777000
            case ["INTTOCHAR"]:
                opcode = 0x170c000
            case ["CHARTOINT"]:
                opcode = 0xc701000
            case ["FALL"]:
                opcode = 0xfa11000
            case _:
                raise ValueError(f"Couldn't parse line {line}")
        os.write(1, opcode.to_bytes(8, "little") + immediate)
    os.write(1, 0xd0d0000.to_bytes(8, "little") + DEFAULT_IMMEDIATE)

if __name__ == "__main__":
    main()
