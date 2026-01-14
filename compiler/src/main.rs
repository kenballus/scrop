use std::{
    collections::HashMap,
    io::{Read, stdin},
    str::from_utf8,
};

#[derive(Debug)]
enum Expression<'a> {
    Int(u64),
    Bool(bool),
    Char(u8),
    Symbol(&'a [u8]),
    PrimitiveFnCall(PrimitiveFnName, Vec<Expression<'a>>),
    Null,
    If(Vec<Expression<'a>>),
    Let(Vec<(&'a [u8], Expression<'a>)>, Vec<Expression<'a>>),
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
    NaryAllPairs(usize),         // implementation_arity
    NaryFold(usize, usize, u64), // implementation_arity, min_args, default_argument
}

impl PrimitiveFnName {
    fn arity(&self) -> PrimitiveFnArity {
        match self {
            PrimitiveFnName::Add1 => PrimitiveFnArity::Unary,
            PrimitiveFnName::Sub1 => PrimitiveFnArity::Unary,
            PrimitiveFnName::Add => PrimitiveFnArity::NaryFold(2, 0, 0),
            PrimitiveFnName::Sub => PrimitiveFnArity::NaryFold(2, 1, 0),
            PrimitiveFnName::Mul => PrimitiveFnArity::NaryFold(2, 0, 1),
            PrimitiveFnName::Lt => PrimitiveFnArity::NaryAllPairs(2),
            PrimitiveFnName::Eq => PrimitiveFnArity::NaryAllPairs(2),
            PrimitiveFnName::EqP => PrimitiveFnArity::NaryAllPairs(2),
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

fn is_delimiter(v: u8) -> bool {
    v.is_ascii_whitespace() || matches!(v, b'(' | b')')
}

fn starts_with_delimiter(input: &[u8]) -> bool {
    input.is_empty() || is_delimiter(input[0])
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

    let (symbol, input) = input.split_at(bytes_consumed);
    if starts_with_delimiter(input) {
        Some((symbol, input))
    } else {
        None
    }
}

fn consume_character(input: &[u8]) -> Option<(u8, &[u8])> {
    if let Some(input) = consume_bytes(input, b"#\\") {
        if !input.is_empty() && is_ascii_printable(input[0]) && starts_with_delimiter(&input[1..]) {
            Some((input[0], &input[1..]))
        } else {
            None
        }
    } else {
        None
    }
}

fn consume_null(input: &[u8]) -> Option<&[u8]> {
    if let Some(input) = consume_bytes(input, b"'()") {
        Some(input)
    } else {
        None
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
    input: &'a [u8],
) -> Option<(PrimitiveFnName, Vec<Expression<'a>>, &'a [u8])> {
    if let Some(input) = consume_bytes(input, b"(") {
        if let Some((sym, input)) = consume_symbol(input) {
            let (args, input) = consume_expressions(consume_whitespace(input));
            if let Some(input) = consume_bytes(consume_whitespace(input), b")") {
                if let Some(name) = parse_primitive_fn_name(sym) {
                    Some((name, args, input))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn consume_if<'a>(input: &'a [u8]) -> Option<(Vec<Expression<'a>>, &'a [u8])> {
    if let Some(input) = consume_bytes(input, b"(") {
        if let Some(input) = consume_bytes(consume_whitespace(input), b"if") {
            let (args, input) = consume_expressions(consume_whitespace(input));
            if let Some(input) = consume_bytes(consume_whitespace(input), b")") {
                if matches!(args.len(), 2 | 3) {
                    Some((args, input))
                } else {
                    panic!("Invalid argument count to if")
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn consume_bytes<'a>(input: &'a [u8], pattern: &'a [u8]) -> Option<&'a [u8]> {
    if input.starts_with(pattern) {
        Some(&input[pattern.len()..])
    } else {
        None
    }
}

fn consume_binding_list<'a>(input: &'a [u8]) -> (Vec<(&'a [u8], Expression<'a>)>, &'a [u8]) {
    if let Some(mut input) = consume_bytes(input, b"(") {
        let mut bindings = Vec::new();
        loop {
            if let Some(new_input) = consume_bytes(consume_whitespace(input), b"(") {
                if let Some((symbol, new_input)) = consume_symbol(consume_whitespace(new_input)) {
                    if let Some((exp, new_input)) =
                        consume_expression(consume_whitespace(new_input))
                    {
                        if let Some(new_input) = consume_bytes(consume_whitespace(new_input), b")")
                        {
                            input = new_input;
                            bindings.push((symbol, exp));
                        } else {
                            panic!("Unexpected data after expression in binding list!")
                        }
                    } else {
                        panic!("Couldn't parse expression in binding list!")
                    }
                } else {
                    panic!("Couldn't parse symbol in binding list!")
                }
            } else if let Some(input) = consume_bytes(consume_whitespace(input), b")") {
                return (bindings, input);
            } else {
                panic!("Couldn't find '(' in binding list entry!")
            }
        }
    } else {
        panic!("Couldn't find '(' in binding list!")
    }
}

fn consume_let<'a>(
    input: &'a [u8],
) -> Option<(
    Vec<(&'a [u8], Expression<'a>)>,
    Vec<Expression<'a>>,
    &'a [u8],
)> {
    if let Some(input) = consume_bytes(input, b"(") {
        if let Some(input) = consume_bytes(consume_whitespace(input), b"let") {
            let (bindings, input) = consume_binding_list(consume_whitespace(input));
            let (exps, input) = consume_expressions(consume_whitespace(input));
            if let Some(input) = consume_bytes(consume_whitespace(input), b")") {
                Some((bindings, exps, input))
            } else {
                panic!("Couldn't find closing ')' for let expression")
            }
        } else {
            None
        }
    } else {
        None
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
    let input = &input[bytes_consumed..];
    if starts_with_delimiter(input) {
        Some((result, input))
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
    let input = &input[BOOL_LITERAL_LEN..];
    if starts_with_delimiter(input) {
        Some((result, input))
    } else {
        None
    }
}

fn consume_whitespace(input: &[u8]) -> &[u8] {
    if input.is_empty() || !input[0].is_ascii_whitespace() {
        input
    } else {
        consume_whitespace(&input[1..])
    }
}

fn consume_expression<'a>(input: &'a [u8]) -> Option<(Expression<'a>, &'a [u8])> {
    if let Some((v, input)) = consume_int(input) {
        Some((Expression::Int(v), input))
    } else if let Some((v, input)) = consume_bool(input) {
        Some((Expression::Bool(v), input))
    } else if let Some((v, input)) = consume_if(input) {
        Some((Expression::If(v), input))
    } else if let Some((name, args, input)) = consume_primitive_fn_call(input) {
        Some((Expression::PrimitiveFnCall(name, args), input))
    } else if let Some((v, input)) = consume_character(input) {
        Some((Expression::Char(v), input))
    } else if let Some(input) = consume_null(input) {
        Some((Expression::Null, input))
    } else if let Some((sym, input)) = consume_symbol(input) {
        Some((Expression::Symbol(sym), input))
    } else if let Some((bindings, exps, input)) = consume_let(input) {
        Some((Expression::Let(bindings, exps), input))
    } else {
        None
    }
}

fn consume_expressions<'a>(mut input: &'a [u8]) -> (Vec<Expression<'a>>, &'a [u8]) {
    let mut result = Vec::new();
    while !input.is_empty()
        && let Some((exp, new_input)) = consume_expression(input)
    {
        result.push(exp);
        input = consume_whitespace(new_input);
    }
    (result, input)
}

fn lower_expression<'a>(
    exp: Expression<'a>,
    env: HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    match exp {
        Expression::Int(x) => result.push("LOAD64 ".to_owned() + &x.to_string()),
        Expression::Char(x) => result.push("LOAD64 #\\".to_owned() + format!("x{:x}", x).as_str()),
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
                        panic!("incorrect argument count for unary primitive function {name:?}");
                    }
                    for arg in args {
                        result.append(&mut lower_expression(arg, env.clone(), stack_slots_used));
                    }
                    result.push(name_string)
                }
                PrimitiveFnArity::NaryAllPairs(implementation_arity) => {
                    let mut stack_slots_used = stack_slots_used;
                    if args.len() < implementation_arity {
                        for arg in args.into_iter() {
                            result.append(&mut lower_expression(
                                arg,
                                env.clone(),
                                stack_slots_used,
                            ));
                            result.push("FORGET".to_owned());
                        }
                        result.append(&mut lower_expression(
                            Expression::Bool(true),
                            env.clone(),
                            stack_slots_used,
                        ));
                    } else {
                        let num_args: usize = args.len();
                        for arg in args {
                            result.append(&mut lower_expression(
                                arg,
                                env.clone(),
                                stack_slots_used,
                            ));
                            stack_slots_used += 1;
                        }
                        // From this point forward, stack_slots_used is not updated, even though
                        // the stack is used. This is because we don't call lower_expression again
                        // in this match arm, so it would be a dead store.
                        for (i, j) in (0..num_args).zip(1..num_args) {
                            result.append(&mut vec![
                                "GET ".to_owned() + &i.to_string(),
                                "GET ".to_owned() + &j.to_string(),
                                "LT".to_owned(),
                            ]);
                            if i != 0 {
                                result.push("AND".to_owned());
                            }
                        }
                        for _ in 0..num_args {
                            result.push("FALL".to_owned());
                        }
                    }
                }
                PrimitiveFnArity::NaryFold(implementation_arity, min_args, default_argument) => {
                    if args.len() < min_args {
                        panic!("Too few arguments provided to {name:?}");
                    }
                    while args.len() < implementation_arity {
                        args.insert(0, Expression::Int(default_argument));
                    }
                    let mut stack_slots_used = stack_slots_used;
                    for (i, arg) in args.into_iter().enumerate() {
                        result.append(&mut lower_expression(arg, env.clone(), stack_slots_used));
                        stack_slots_used += 1; // arg
                        if (i == implementation_arity - 1)
                            || (i >= implementation_arity
                                && ((i % (implementation_arity - 1)) == 0))
                        {
                            result.push(name_string.clone());
                            // Note: this cannot be rewritten as
                            // `stack_slots_used -= 1 - implementation_arity`
                            // because that will promote 1 to usize, and then underflow.
                            stack_slots_used -= implementation_arity; // implementation args
                            stack_slots_used += 1; //                    implementation result
                        }
                    }
                }
            }
        }
        Expression::Null => result.push("LOAD64 NULL".to_owned()),
        Expression::If(mut v) => {
            let mut stack_slots_used = stack_slots_used;
            // cond
            result.append(&mut lower_expression(
                v.remove(0),
                env.clone(),
                stack_slots_used,
            ));
            stack_slots_used += 1; // cond
            result.push("LOAD64 #f".to_owned());
            stack_slots_used += 1; // load
            result.push("EQP".to_owned());
            stack_slots_used -= 1; // eqp

            // cons
            let mut cons_code = lower_expression(v.remove(0), env.clone(), stack_slots_used);

            // alt
            let mut alt_code = if let Some(alt_code) = v.pop() {
                lower_expression(alt_code, env.clone(), stack_slots_used)
            } else {
                vec!["LOAD64 UNSPECIFIED".to_owned()]
            };

            cons_code.push("JUMP ".to_owned() + &alt_code.len().to_string());

            result.push("CJUMP ".to_owned() + &cons_code.len().to_string());
            result.append(&mut cons_code);
            result.append(&mut alt_code);
        }
        Expression::Let(bindings, exps) => {
            let mut new_env = env.clone();
            let mut stack_slots_used = stack_slots_used;
            let num_bindings = bindings.len();
            for (name, exp) in bindings.into_iter() {
                if new_env.insert(name, stack_slots_used).is_some() {
                    panic!("Duplicate key in let binding");
                }
                result.append(&mut lower_expression(exp, env.clone(), stack_slots_used));
                stack_slots_used += 1;
            }
            result.append(&mut lower_expressions(exps, new_env, stack_slots_used));
            for _ in 0..num_bindings {
                result.push("FALL".to_owned());
            }
        }
        Expression::Symbol(name) => {
            if let Some(env_index) = env.get(name) {
                result.push("GET ".to_owned() + &env_index.to_string());
            } else {
                panic!(
                    "Couldn't find environment entry for \"{}\"",
                    from_utf8(name).unwrap()
                )
            }
        }
    };
    result
}

fn lower_expressions<'a>(
    exps: Vec<Expression<'a>>,
    env: HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    let num_exps = exps.len();
    for (i, exp) in exps.into_iter().enumerate() {
        result.append(&mut lower_expression(exp, env.clone(), stack_slots_used));
        if i != num_exps - 1 {
            result.push("FORGET".to_owned())
        }
    }
    result
}

fn compile_all(input_slice: &[u8]) -> Vec<String> {
    let (ast, input_slice) = consume_expressions(consume_whitespace(input_slice));
    // dbg!(&ast);
    if !input_slice.is_empty() {
        panic!("Leftover data: {:?}", input_slice);
    }
    lower_expressions(ast, HashMap::new(), 0)
}

#[test]
#[should_panic]
fn let_binding_too_many_args() {
    compile_all(b"(let ((x 1 1)) x)");
}

#[test]
#[should_panic]
fn let_binding_list_not_nested() {
    compile_all(b"(let (x 1) x)");
}

fn main() {
    let mut input_vec = Vec::new();
    let _bytes_read = stdin().read_to_end(&mut input_vec);
    println!("{}", compile_all(&input_vec[..]).join("\n"))
}
