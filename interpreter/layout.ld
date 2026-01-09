SECTIONS {
    . = 0x8008000;
    .text : {
    }

    . = 0xadd1000;
    .text.add1 : {
        *(add1)
    }

    . = 0x50b1000;
    .text.sub1 : {
        *(sub1)
    }

    . = 0xd0d0000;
    .text.done : {
        *(done)
    }

    . = 0x10ad000;
    .text.load64 : {
        *(load64)
    }

    . = 0x0add000;
    .text.add : {
        *(add)
    }

    . = 0x050b000;
    .text.sub : {
        *(sub)
    }

    . = 0x0a55000;
    .text.mul : {
        *(mul)
    }

    . = 0x1001000;
    .text.lt : {
        *(lt)
    }

    . = 0xe3e3000;
    .text.eq : {
        *(eq)
    }

    . = 0xeeee000;
    .text.zeroq : {
        *(zeroq)
    }

    . = 0x1234000;
    .text.integerq : {
        *(integerq)
    }

    . = 0xb001000;
    .text.booleanq : {
        *(booleanq)
    }

    . = 0x4321000;
    .text.nullq : {
        *(nullq)
    }

    . = 0x7777000;
    .text.not : {
        *(not)
    }

    . = 0xcaca000;
    .text.charq : {
        *(charq)
    }

    . = 0xc701000;
    .text.chartoint : {
        *(chartoint)
    }

    . = 0x170c000;
    .text.inttochar : {
        *(inttochar)
    }
}
