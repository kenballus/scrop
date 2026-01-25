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
    Null,
    Form(Vec<Expression<'a>>),
    String(Vec<u8>),
}

fn is_delimiter(v: u8) -> bool {
    v.is_ascii_whitespace() || matches!(v, b'(' | b')' | b';')
}

fn starts_with_delimiter(input: &[u8]) -> bool {
    input.is_empty() || is_delimiter(input[0])
}

fn is_symbol_start_char(v: u8) -> bool {
    v.is_ascii_alphabetic()
        || v.is_ascii_digit()
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
    is_symbol_start_char(v) || v == b'#'
}

fn consume_string(input: &[u8]) -> Option<(Vec<u8>, &[u8])> {
    if let Some(mut input) = consume_bytes(input, b"\"") {
        let mut result = Vec::new();
        loop {
            if let Some(new_input) = consume_bytes(input, b"\\\\") {
                result.push(b'\\');
                input = new_input;
            } else if let Some(new_input) = consume_bytes(input, b"\\n") {
                result.push(b'\n');
                input = new_input;
            } else if let Some(new_input) = consume_bytes(input, b"\\t") {
                result.push(b'\t');
                input = new_input;
            } else if let Some(new_input) = consume_bytes(input, b"\\\"") {
                result.push(b'"');
                input = new_input;
            } else if let Some(new_input) = consume_bytes(input, b"\\") {
                result.push(b'\n');
                input = new_input;
            } else if let Some(new_input) = consume_bytes(input, b"\"") {
                input = new_input;
                break;
            } else if input.starts_with(b"\\") {
                panic!("Unrecognized escape sequence in string literal!");
            } else {
                result.push(input[0]);
                input = &input[1..];
            }
        }
        Some((result, input))
    } else {
        None
    }
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
    if starts_with_delimiter(input) && symbol.iter().any(|c| !c.is_ascii_digit()) {
        Some((symbol, input))
    } else {
        None
    }
}

fn consume_character(input: &[u8]) -> Option<(u8, &[u8])> {
    if let Some(input) = consume_bytes(input, b"#\\") {
        if !input.is_empty() && starts_with_delimiter(&input[1..]) {
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

fn consume_form(input: &[u8]) -> Option<(Vec<Expression<'_>>, &[u8])> {
    if let Some(input) = consume_bytes(input, b"(") {
        let (args, input) = consume_expressions(consume_whitespace(input));
        if let Some(input) = consume_bytes(consume_whitespace(input), b")") {
            Some((args, input))
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

fn consume_int(input: &[u8]) -> Option<(u64, &[u8])> {
    let mut result: u64 = 0;
    let mut bytes_consumed: usize = 0;
    while bytes_consumed < input.len() && input[bytes_consumed].is_ascii_digit() {
        result *= 10;
        result += u64::from(input[bytes_consumed] - b'0');
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

fn consume_line_comment(input: &[u8]) -> Option<&[u8]> {
    if let Some(mut input) = consume_bytes(input, b";") {
        loop {
            if input.is_empty() || input.starts_with(b"\n") {
                return Some(input);
            }
            input = &input[1..];
        }
    } else {
        None
    }
}

fn consume_nested_comment(input: &[u8]) -> Option<&[u8]> {
    if let Some(mut input) = consume_bytes(input, b"#|") {
        loop {
            if input.is_empty() {
                return None;
            }
            if let Some(input) = consume_bytes(input, b"|#") {
                return Some(input);
            }
            if input.starts_with(b"#|") {
                if let Some(new_input) = consume_nested_comment(input) {
                    input = new_input;
                } else {
                    return None;
                }
            } else {
                input = &input[1..];
            }
        }
    } else {
        None
    }
}

fn consume_datum_comment(input: &[u8]) -> Option<&[u8]> {
    if let Some(input) = consume_bytes(input, b"#;") {
        if let Some((_, input)) = consume_expression(consume_whitespace(input)) {
            Some(input)
        } else {
            None
        }
    } else {
        None
    }
}

fn consume_whitespace(input: &[u8]) -> &[u8] {
    if input.is_empty() {
        input
    } else if input[0].is_ascii_whitespace() {
        consume_whitespace(&input[1..])
    } else if let Some(input) = consume_line_comment(input) {
        consume_whitespace(input)
    } else if let Some(input) = consume_nested_comment(input) {
        consume_whitespace(input)
    } else if let Some(input) = consume_datum_comment(input) {
        consume_whitespace(input)
    } else {
        input
    }
}

fn consume_expression(input: &[u8]) -> Option<(Expression<'_>, &[u8])> {
    if let Some((v, input)) = consume_int(input) {
        Some((Expression::Int(v), input))
    } else if let Some((v, input)) = consume_bool(input) {
        Some((Expression::Bool(v), input))
    } else if let Some((v, input)) = consume_character(input) {
        Some((Expression::Char(v), input))
    } else if let Some(input) = consume_null(input) {
        Some((Expression::Null, input))
    } else if let Some((sym, input)) = consume_symbol(input) {
        Some((Expression::Symbol(sym), input))
    } else if let Some((args, input)) = consume_form(input) {
        Some((Expression::Form(args), input))
    } else if let Some((v, input)) = consume_string(input) {
        Some((Expression::String(v), input))
    } else {
        None
    }
}

fn consume_expressions(mut input: &[u8]) -> (Vec<Expression<'_>>, &[u8]) {
    let mut result = Vec::new();
    while !input.is_empty()
        && let Some((exp, new_input)) = consume_expression(input)
    {
        result.push(exp);
        input = consume_whitespace(new_input);
    }
    (result, input)
}

fn lower_let<'a>(
    mut args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    if let Expression::Form(bindings) = args.remove(0) {
        let mut new_bindings = HashMap::new();
        let mut stack_slots_used = stack_slots_used;
        let num_bindings = bindings.len();

        for binding in bindings {
            if let Expression::Form(mut binding) = binding {
                assert!(
                    binding.len() == 2,
                    "let binding has incorrect argument count."
                );
                if let (Expression::Symbol(name), exp) = (binding.remove(0), binding.remove(0)) {
                    let insert_rc = new_bindings.insert(name, stack_slots_used);
                    assert!(insert_rc.is_none(), "Duplicate key in let binding");
                    result.append(&mut lower_expression(exp, &env.clone(), stack_slots_used));
                    stack_slots_used += 1;
                } else {
                    panic!("let binding args are not (Symbol, Expr)")
                }
            } else {
                panic!("let binding is not a form")
            }
        }

        let new_env = &mut env.clone();
        new_env.extend(new_bindings.drain());
        result.append(&mut lower_expressions(args, new_env, stack_slots_used));
        result.push(format!("FALL {num_bindings}"));
    } else {
        panic!("let bindings is not a form")
    }
    result
}

fn lower_begin<'a>(
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    if args.is_empty() {
        // Technically wrong; whether begin allows 0 args is context-dependent
        vec!["LOAD UNSPECIFIED".to_owned()]
    } else {
        lower_expressions(args, env, stack_slots_used)
    }
}

fn lower_if<'a>(
    mut args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut stack_slots_used = stack_slots_used;
    assert!(matches!(args.len(), 2 | 3), "Invalid argument count to if");
    // cond
    result.append(&mut lower_expression(
        args.remove(0),
        &env.clone(),
        stack_slots_used,
    ));
    stack_slots_used += 1; // cond
    result.push("LOAD #f".to_owned());
    stack_slots_used += 1; // load
    result.push("EQP 2".to_owned());
    stack_slots_used -= 1; // eqp

    // consequent
    let mut consequent_code = lower_expression(args.remove(0), &env.clone(), stack_slots_used);

    // alternative
    let mut alternative_code = if let Some(alternative_code) = args.pop() {
        lower_expression(alternative_code, &env.clone(), stack_slots_used)
    } else {
        vec!["LOAD UNSPECIFIED".to_owned()]
    };

    consequent_code.push("JUMP ".to_owned() + &alternative_code.len().to_string());

    result.push("CJUMP ".to_owned() + &consequent_code.len().to_string());
    result.append(&mut consequent_code);
    result.append(&mut alternative_code);
    result
}

fn lower_nary_primitive<'a>(
    mnemonic: &str,
    n: usize,
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    assert!(
        args.len() == n,
        "incorrect argument count for {n}-ary primitive"
    );
    for arg in args {
        result.append(&mut lower_expression(arg, &env.clone(), stack_slots_used));
    }
    result.push(mnemonic.to_owned());
    result
}

fn lower_variadic_primitive<'a>(
    min_args: usize,
    mnemonic: &str,
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    let num_args = args.len();
    assert!(
        num_args >= min_args,
        "Too few arguments provided to variadic primitive"
    );
    for (i, arg) in args.into_iter().rev().enumerate() {
        result.append(&mut lower_expression(
            arg,
            &env.clone(),
            stack_slots_used + i,
        ));
    }
    result.push(format!("{mnemonic} {num_args}"));
    result
}

fn lower_form<'a>(
    mut args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    assert!(!args.is_empty(), "Empty form!");
    if let Expression::Symbol(name) = args.remove(0) {
        if env.contains_key(name) {
            todo!("Function calls are not yet implemented.")
        }
        match name {
            b"begin" => lower_begin(args, env, stack_slots_used),
            b"let" => lower_let(args, env, stack_slots_used),
            b"if" => lower_if(args, env, stack_slots_used),
            b"add1" => lower_nary_primitive("ADD1", 1, args, env, stack_slots_used),
            b"sub1" => lower_nary_primitive("SUB1", 1, args, env, stack_slots_used),
            b"zero?" => lower_nary_primitive("ZEROP", 1, args, env, stack_slots_used),
            b"integer?" => lower_nary_primitive("INTEGERP", 1, args, env, stack_slots_used),
            b"boolean?" => lower_nary_primitive("BOOLEANP", 1, args, env, stack_slots_used),
            b"char?" => lower_nary_primitive("CHARP", 1, args, env, stack_slots_used),
            b"null?" => lower_nary_primitive("NULLP", 1, args, env, stack_slots_used),
            b"not" => lower_nary_primitive("NOT", 1, args, env, stack_slots_used),
            b"char->integer" => lower_nary_primitive("CHARTOINT", 1, args, env, stack_slots_used),
            b"integer->char" => lower_nary_primitive("INTTOCHAR", 1, args, env, stack_slots_used),
            b"+" => lower_variadic_primitive(0, "ADD", args, env, stack_slots_used),
            b"-" => lower_variadic_primitive(1, "SUB", args, env, stack_slots_used),
            b"*" => lower_variadic_primitive(0, "MUL", args, env, stack_slots_used),
            b"<" => lower_variadic_primitive(0, "LT", args, env, stack_slots_used),
            b"=" => lower_variadic_primitive(0, "EQ", args, env, stack_slots_used),
            b"eq?" => lower_variadic_primitive(0, "EQP", args, env, stack_slots_used),
            b"string" => lower_variadic_primitive(0, "STRING", args, env, stack_slots_used),
            b"string-append" => {
                lower_variadic_primitive(0, "STRINGAPPEND", args, env, stack_slots_used)
            }
            b"string-ref" => lower_nary_primitive("STRINGREF", 2, args, env, stack_slots_used),
            b"string-set!" => lower_nary_primitive("STRINGSET", 3, args, env, stack_slots_used),
            b"vector" => lower_variadic_primitive(0, "VECTOR", args, env, stack_slots_used),
            b"vector-append" => {
                lower_variadic_primitive(0, "VECTORAPPEND", args, env, stack_slots_used)
            }
            b"vector-ref" => lower_nary_primitive("VECTORREF", 2, args, env, stack_slots_used),
            b"vector-set!" => lower_nary_primitive("VECTORSET", 3, args, env, stack_slots_used),
            b"cons" => lower_nary_primitive("CONS", 2, args, env, stack_slots_used),
            b"car" => lower_nary_primitive("CAR", 1, args, env, stack_slots_used),
            b"cdr" => lower_nary_primitive("CDR", 1, args, env, stack_slots_used),
            _ => panic!("Cannot resolve symbol '{name:?}'"),
        }
    } else {
        panic!("First entry in form is invalid.")
    }
}

fn lower_expression<'a>(
    exp: Expression<'a>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    match exp {
        Expression::Int(x) => vec!["LOAD ".to_owned() + &x.to_string()],
        Expression::Char(x) => vec![format!("LOAD #\\x{x:x}")],
        Expression::Bool(x) => vec!["LOAD ".to_owned() + if x { "#t" } else { "#f" }],
        Expression::Form(args) => lower_form(args, env, stack_slots_used),
        Expression::Null => vec!["LOAD NULL".to_owned()],
        Expression::Symbol(name) => {
            if let Some(env_index) = env.get(name) {
                vec!["GET ".to_owned() + &env_index.to_string()]
            } else {
                panic!(
                    "Couldn't find environment entry for \"{}\"",
                    from_utf8(name).unwrap()
                )
            }
        }
        Expression::String(v) => lower_variadic_primitive(
            0,
            "STRING",
            v.into_iter().map(Expression::Char).collect(),
            env,
            stack_slots_used,
        ),
    }
}

fn lower_expressions<'a>(
    exps: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    let num_exps = exps.len();
    for (i, exp) in exps.into_iter().enumerate() {
        result.append(&mut lower_expression(exp, &env.clone(), stack_slots_used));
        if i != num_exps - 1 {
            result.push("FORGET".to_owned());
        }
    }
    result
}

fn compile_all(input_slice: &[u8]) -> Vec<String> {
    let (ast, input_slice) = consume_expressions(consume_whitespace(input_slice));
    // dbg!(&ast);
    assert!(
        input_slice.is_empty(),
        "Parsing failed. Leftover data: {input_slice:?}"
    );
    lower_expressions(ast, &HashMap::new(), 0)
}

fn main() {
    let mut input_vec = Vec::new();
    let _bytes_read = stdin().read_to_end(&mut input_vec);
    println!("{}", compile_all(&input_vec[..]).join("\n"));
}

#[test]
#[should_panic(expected = "let bindings is not a form")]
fn invalid_let_binding_list() {
    compile_all(b"(let 1 1)");
}

#[test]
#[should_panic(expected = "let binding is not a form")]
fn invalid_let_binding_list_entry() {
    compile_all(b"(let (1) 1)");
}

#[test]
#[should_panic(expected = "let binding has incorrect argument count.")]
fn let_binding_too_many_args() {
    compile_all(b"(let ((x 1 1)) x)");
}

#[test]
#[should_panic(expected = "Duplicate key in let binding")]
fn let_binding_duplicate_key() {
    compile_all(b"(let ((x 1) (x 1)) x)");
}

#[test]
#[should_panic(expected = "let binding is not a form")]
fn let_binding_list_not_nested() {
    compile_all(b"(let (x 1) x)");
}

#[test]
#[should_panic(expected = "Invalid argument count to if")]
fn too_few_if_args() {
    compile_all(b"(if)");
}

#[test]
#[should_panic(expected = "Invalid argument count to if")]
fn too_many_if_args() {
    compile_all(b"(if 1 2 3 4)");
}

#[test]
#[should_panic(expected = "Parsing failed. Leftover data: [93]")]
fn leftover_data() {
    compile_all(b"]");
}

#[test]
#[should_panic(expected = "incorrect argument count for 1-ary primitive")]
fn too_few_unary_args() {
    compile_all(b"(not)");
}

#[test]
#[should_panic(expected = "incorrect argument count for 1-ary primitive")]
fn too_many_unary_args() {
    compile_all(b"(not 1 2)");
}

#[test]
#[should_panic(expected = "Too few arguments provided to variadic primitive")]
fn too_few_variadic_args() {
    compile_all(b"(-)");
}

#[test]
#[should_panic(expected = "Couldn't find environment entry for \"a\"")]
fn use_undefined_variable() {
    compile_all(b"a");
}

#[test]
#[should_panic(expected = "Parsing failed. Leftover data: [35, 124, 32, 35, 124, 32, 124, 35]")]
fn mismatched_nested_comment() {
    compile_all(b"#| #| |#");
}

#[test]
#[should_panic(expected = "let binding args are not (Symbol, Expr)")]
fn numeric_symbol() {
    compile_all(b"(let ((1 0)) 1)");
}
