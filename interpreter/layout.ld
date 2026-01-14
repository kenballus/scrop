SECTIONS {
    . = 0x8008000;
    .text : {
    }

    . = 0xc001000;
    .text.and : {
        *(and)
    }

    . = 0xfa11000;
    .text.fall : {
        *(fall)
    }

    . = 0x3e3e000;
    .text.eqp : {
        *(eqp)
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
    .text.zerop : {
        *(zerop)
    }

    . = 0x1234000;
    .text.integerp : {
        *(integerp)
    }

    . = 0xb001000;
    .text.booleanp : {
        *(booleanp)
    }

    . = 0x4321000;
    .text.nullp : {
        *(nullp)
    }

    . = 0x7777000;
    .text.not : {
        *(not)
    }

    . = 0xcaca000;
    .text.charp : {
        *(charp)
    }

    . = 0xc701000;
    .text.chartoint : {
        *(chartoint)
    }

    . = 0x170c000;
    .text.inttochar : {
        *(inttochar)
    }

    . = 0x70ad000;
    .text.jump : {
        *(jump)
    }

    . = 0xca7000;
    .text.cjump : {
        *(cjump)
    }

    . = 0x9e7000;
    .text.get : {
        *(get)
    }

    . = 0x49e7000;
    .text.forget : {
        *(forget)
    }

}
