use std::io::{Read, stdin};

#[derive(Debug)]
enum Expression<'a> {
    Int(u64),
    Bool(bool),
    Char(u8),
    Symbol(&'a [u8]),
    PrimitiveFnCall(PrimitiveFnName, Vec<Expression<'a>>),
    Null,
    If(Vec<Expression<'a>>),
}

#[derive(Debug)]
enum PrimitiveFnName {
    Add1,
    Sub1,
    Add,
    Sub,
    Mul,
    Lt,
    Eq,
    EqP,
    ZeroP,
    IntegerP,
    BooleanP,
    CharP,
    NullP,
    Not,
    CharToInt,
    IntToChar,
}

enum PrimitiveFnArity {
    Unary,
    NaryWithDefaultBoolResult(usize, bool), // implementation_arity, default_result
    NaryWithDefaultIntInput(usize, u64),    // implementation_arity, default_input
}

impl PrimitiveFnName {
    fn arity(&self) -> PrimitiveFnArity {
        match self {
            PrimitiveFnName::Add1 => PrimitiveFnArity::Unary,
            PrimitiveFnName::Sub1 => PrimitiveFnArity::Unary,
            PrimitiveFnName::Add => PrimitiveFnArity::NaryWithDefaultIntInput(2, 0),
            PrimitiveFnName::Sub => PrimitiveFnArity::NaryWithDefaultIntInput(2, 0),
            PrimitiveFnName::Mul => PrimitiveFnArity::NaryWithDefaultIntInput(2, 1),
            PrimitiveFnName::Lt => PrimitiveFnArity::NaryWithDefaultBoolResult(2, true),
            PrimitiveFnName::Eq => PrimitiveFnArity::NaryWithDefaultBoolResult(2, true),
            PrimitiveFnName::EqP => PrimitiveFnArity::NaryWithDefaultBoolResult(2, true),
            PrimitiveFnName::ZeroP => PrimitiveFnArity::Unary,
            PrimitiveFnName::IntegerP => PrimitiveFnArity::Unary,
            PrimitiveFnName::BooleanP => PrimitiveFnArity::Unary,
            PrimitiveFnName::CharP => PrimitiveFnArity::Unary,
            PrimitiveFnName::NullP => PrimitiveFnArity::Unary,
            PrimitiveFnName::Not => PrimitiveFnArity::Unary,
            PrimitiveFnName::CharToInt => PrimitiveFnArity::Unary,
            PrimitiveFnName::IntToChar => PrimitiveFnArity::Unary,
        }
    }
}

fn check_for_delimiter(input: &[u8]) -> bool {
    input.is_empty() || input[0].is_ascii_whitespace() || matches!(input[0], b')' | b'(')
}

fn is_ascii_printable(v: u8) -> bool {
    v.is_ascii_alphanumeric() || v.is_ascii_punctuation()
}

fn is_symbol_start_char(v: u8) -> bool {
    v.is_ascii_alphabetic()
        || matches!(
            v,
            b'-' | b'+'
                | b'='
                | b'_'
                | b'*'
                | b'&'
                | b'^'
                | b'%'
                | b'$'
                | b'!'
                | b'~'
                | b':'
                | b'|'
                | b'\\'
                | b'?'
                | b'/'
                | b'<'
                | b'>'
        )
}

fn is_symbol_char(v: u8) -> bool {
    is_symbol_start_char(v) || v.is_ascii_digit()
}

fn consume_symbol(input: &[u8]) -> Option<(&[u8], &[u8])> {
    if input.is_empty() || !is_symbol_start_char(input[0]) {
        return None;
    }
    let mut bytes_consumed: usize = 1;

    while bytes_consumed < input.len() && is_symbol_char(input[bytes_consumed]) {
        bytes_consumed += 1;
    }
    let remaining_input = &input[bytes_consumed..];
    if check_for_delimiter(remaining_input) {
        Some((&input[..bytes_consumed], remaining_input))
    } else {
        None
    }
}

fn consume_character(input: &[u8]) -> Option<(u8, &[u8])> {
    if !input.starts_with(b"#\\") {
        return None;
    }
    if is_ascii_printable(input[2]) && check_for_delimiter(&input[3..]) {
        Some((input[2], &input[3..]))
    } else {
        None
    }
}

fn consume_null(input: &[u8]) -> Option<&[u8]> {
    if !input.starts_with(b"'()") {
        None
    } else {
        Some(&input[3..])
    }
}

fn parse_primitive_fn_name(input: &[u8]) -> Option<PrimitiveFnName> {
    match input {
        b"add1" => Some(PrimitiveFnName::Add1),
        b"sub1" => Some(PrimitiveFnName::Sub1),
        b"+" => Some(PrimitiveFnName::Add),
        b"-" => Some(PrimitiveFnName::Sub),
        b"*" => Some(PrimitiveFnName::Mul),
        b"<" => Some(PrimitiveFnName::Lt),
        b"=" => Some(PrimitiveFnName::Eq),
        b"zero?" => Some(PrimitiveFnName::ZeroP),
        b"integer?" => Some(PrimitiveFnName::IntegerP),
        b"boolean?" => Some(PrimitiveFnName::BooleanP),
        b"char?" => Some(PrimitiveFnName::CharP),
        b"null?" => Some(PrimitiveFnName::NullP),
        b"not" => Some(PrimitiveFnName::Not),
        b"char->integer" => Some(PrimitiveFnName::CharToInt),
        b"integer->char" => Some(PrimitiveFnName::IntToChar),
        b"eq?" => Some(PrimitiveFnName::EqP),
        _ => None,
    }
}

fn consume_primitive_fn_call<'a>(
    mut input: &'a [u8],
) -> Option<((PrimitiveFnName, Vec<Expression<'a>>), &'a [u8])> {
    if input.is_empty() || !input.starts_with(b"(") {
        return None;
    }
    input = &input[1..];
    if let Some((sym, input)) = consume_symbol(input) {
        let (args, input) = consume_expressions(input);
        let input = consume_whitespace(input);
        if input.is_empty() || !input.starts_with(b")") {
            None
        } else if let Some(name) = parse_primitive_fn_name(sym) {
            Some(((name, args), &input[1..]))
        } else {
            None
        }
    } else {
        panic!("Invalid function invocation")
    }
}

fn consume_if<'a>(input: &'a [u8]) -> Option<(Vec<Expression<'a>>, &'a [u8])> {
    if input.is_empty() || !input.starts_with(b"(") {
        return None;
    }
    let input = &input[1..];
    if let Some((sym, input)) = consume_symbol(input) {
        let (args, input) = consume_expressions(input);
        let input = consume_whitespace(input);
        if input.is_empty() || !input.starts_with(b")") || sym != b"if" {
            None
        } else if matches!(args.len(), 2|3) {
            Some(((args), &input[1..]))
        } else {
            panic!("Invalid argument count to if")
        }
    } else {
        panic!("Invalid function invocation")
    }
}

fn consume_int(input: &[u8]) -> Option<(u64, &[u8])> {
    let mut result: u64 = 0;
    let mut bytes_consumed: usize = 0;
    while bytes_consumed < input.len() && input[bytes_consumed].is_ascii_digit() {
        result *= 10;
        result += (input[bytes_consumed] - b'0') as u64;
        bytes_consumed += 1;
    }
    if bytes_consumed == 0 {
        return None;
    }
    let remaining_input = &input[bytes_consumed..];
    if check_for_delimiter(remaining_input) {
        Some((result, remaining_input))
    } else {
        None
    }
}

fn consume_bool(input: &[u8]) -> Option<(bool, &[u8])> {
    const BOOL_LITERAL_LEN: usize = 2;
    let result = match input {
        [b'#', b't' | b'T', ..] => true,
        [b'#', b'f' | b'F', ..] => false,
        _ => {
            return None;
        }
    };
    let remaining_input = &input[BOOL_LITERAL_LEN..];
    if check_for_delimiter(remaining_input) {
        Some((result, remaining_input))
    } else {
        None
    }
}

fn consume_whitespace(input: &[u8]) -> &[u8] {
    if input.is_empty() {
        input
    } else {
        let mut bytes_consumed: usize = 0;
        while input[bytes_consumed].is_ascii_whitespace() {
            bytes_consumed += 1;
        }
        &input[bytes_consumed..]
    }
}

fn consume_expression<'a>(mut input: &'a [u8]) -> Option<(Expression<'a>, &'a [u8])> {
    input = consume_whitespace(input);
    if input.is_empty() {
        None
    } else if let Some((v, rem)) = consume_int(input) {
        Some((Expression::Int(v), rem))
    } else if let Some((v, rem)) = consume_bool(input) {
        Some((Expression::Bool(v), rem))
    } else if let Some((v, rem)) = consume_symbol(input) {
        Some((Expression::Symbol(v), rem))
    } else if let Some((v, rem)) = consume_if(input) {
        Some((Expression::If(v), rem))
    } else if let Some(((name, args), rem)) = consume_primitive_fn_call(input) {
        Some((Expression::PrimitiveFnCall(name, args), rem))
    } else if let Some((v, rem)) = consume_character(input) {
        Some((Expression::Char(v), rem))
    } else if let Some(rem) = consume_null(input) {
        Some((Expression::Null, rem))
    } else {
        None
    }
}

fn consume_expressions<'a>(mut input: &'a [u8]) -> (Vec<Expression<'a>>, &'a [u8]) {
    let mut result = Vec::new();
    while !input.is_empty()
        && let Some((exp, remaining_input)) = consume_expression(input)
    {
        result.push(exp);
        input = remaining_input;
    }
    (result, input)
}

fn lower(ast: Vec<Expression<'_>>) -> Vec<String> {
    let mut result = Vec::new();
    for subtree in ast {
        match subtree {
            Expression::Int(x) => result.push("LOAD64 ".to_owned() + &x.to_string()),
            Expression::Char(x) => {
                result.push("LOAD64 #\\".to_owned() + format!("x{:x}", x).as_str())
            }
            Expression::Bool(x) => result.push("LOAD64 ".to_owned() + if x { "#t" } else { "#f" }),
            Expression::PrimitiveFnCall(name, mut args) => {
                let name_string = (match name {
                    PrimitiveFnName::Add1 => "ADD1",
                    PrimitiveFnName::Sub1 => "SUB1",
                    PrimitiveFnName::Add => "ADD",
                    PrimitiveFnName::Sub => "SUB",
                    PrimitiveFnName::Mul => "MUL",
                    PrimitiveFnName::Lt => "LT",
                    PrimitiveFnName::Eq => "EQ",
                    PrimitiveFnName::EqP => "EQP",
                    PrimitiveFnName::ZeroP => "ZEROP",
                    PrimitiveFnName::IntegerP => "INTEGERP",
                    PrimitiveFnName::BooleanP => "BOOLEANP",
                    PrimitiveFnName::CharP => "CHARP",
                    PrimitiveFnName::NullP => "NULLP",
                    PrimitiveFnName::Not => "NOT",
                    PrimitiveFnName::CharToInt => "CHARTOINT",
                    PrimitiveFnName::IntToChar => "INTTOCHAR",
                })
                .to_owned();
                match name.arity() {
                    PrimitiveFnArity::Unary => {
                        if args.len() != 1 {
                            panic!(
                                "incorrect argument count for unary primitive function {name:?}"
                            );
                        }
                        for arg in args {
                            result.append(&mut lower(vec![arg]));
                        }
                        result.push(name_string)
                    }
                    PrimitiveFnArity::NaryWithDefaultBoolResult(
                        implementation_arity,
                        default_result,
                    ) => {
                        if args.len() < implementation_arity {
                            result.append(&mut lower(vec![Expression::Bool(default_result)]));
                        } else {
                            for (i, arg) in args.into_iter().enumerate() {
                                result.append(&mut lower(vec![arg]));
                                if (i == implementation_arity - 1)
                                    || (i >= implementation_arity
                                        && ((i % (implementation_arity - 1)) == 0))
                                {
                                    result.push(name_string.clone());
                                }
                            }
                        }
                    }
                    PrimitiveFnArity::NaryWithDefaultIntInput(
                        implementation_arity,
                        default_input,
                    ) => {
                        while args.len() < implementation_arity {
                            args.insert(0, Expression::Int(default_input));
                        }
                        for (i, arg) in args.into_iter().enumerate() {
                            result.append(&mut lower(vec![arg]));
                            if (i == implementation_arity - 1)
                                || (i >= implementation_arity
                                    && ((i % (implementation_arity - 1)) == 0))
                            {
                                result.push(name_string.clone());
                            }
                        }
                    }
                }
            }
            Expression::Null => result.push("LOAD64 NULL".to_owned()),
            Expression::If(mut v) => {
                // cond
                result.append(&mut lower(vec![v.remove(0)]));
                result.push("LOAD64 #f".to_owned());
                result.push("EQP".to_owned());

                // cons
                let mut cons_code = lower(vec![v.remove(0)]);

                // alt
                let alt_code_opt = match v.pop() {
                    Some(x) => Some(lower(vec![x])),
                    None => None,
                };

                if let Some(ref alt_code) = alt_code_opt {
                    cons_code.push("JUMP ".to_owned() + &alt_code.len().to_string())
                }
                result.push("CJUMP ".to_owned() + &cons_code.len().to_string());
                result.append(&mut cons_code);

                if let Some(mut alt_code) = alt_code_opt {
                    result.append(&mut alt_code);
                }
            }
            Expression::Symbol(_) => todo!(),
        };
    }
    result
}

fn main() {
    let mut input = Vec::new();
    let _bytes_read = stdin().read_to_end(&mut input);
    let (ast, rem) = consume_expressions(&input[..]);
    dbg!(&ast);
    if !rem.is_empty() {
        panic!("Leftover data: {:?}", rem);
    }
    println!("{}", lower(ast).join("\n"))
}
