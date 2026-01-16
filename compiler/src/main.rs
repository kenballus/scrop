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

fn consume_whitespace(input: &[u8]) -> &[u8] {
    if input.is_empty() || !input[0].is_ascii_whitespace() {
        input
    } else {
        consume_whitespace(&input[1..])
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
        let new_env = &mut env.clone();
        let mut stack_slots_used = stack_slots_used;
        let num_bindings = bindings.len();

        for binding in bindings {
            if let Expression::Form(mut binding) = binding {
                assert!(
                    binding.len() == 2,
                    "let binding has incorrect argument count."
                );
                if let (Expression::Symbol(name), exp) = (binding.remove(0), binding.remove(0)) {
                    let insert_rc = new_env.insert(name, stack_slots_used);
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

        result.append(&mut lower_expressions(args, new_env, stack_slots_used));
        for _ in 0..num_bindings {
            result.push("FALL".to_owned());
        }
    } else {
        panic!("let bindings is not a form")
    }
    result
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
    result.push("EQP".to_owned());
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

fn lower_unary_primitive<'a>(
    mnemonic: &str,
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    assert!(
        args.len() == 1,
        "incorrect argument count for unary primitive function"
    );
    for arg in args {
        result.append(&mut lower_expression(arg, &env.clone(), stack_slots_used));
    }
    result.push(mnemonic.to_owned());
    result
}

fn lower_nary_all_pairs_primitive<'a>(
    implementation_arity: usize,
    mnemonic: &str,
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut stack_slots_used = stack_slots_used;
    if args.len() < implementation_arity {
        for arg in args {
            result.append(&mut lower_expression(arg, &env.clone(), stack_slots_used));
            result.push("FORGET".to_owned());
        }
        result.append(&mut lower_expression(
            Expression::Bool(true),
            &env.clone(),
            stack_slots_used,
        ));
    } else {
        let num_args: usize = args.len();
        for arg in args {
            result.append(&mut lower_expression(arg, &env.clone(), stack_slots_used));
            stack_slots_used += 1;
        }
        // From this point forward, stack_slots_used is not updated, even though
        // the stack is used. This is because we don't call lower_expression again
        // in this match arm, so it would be a dead store.
        for (i, j) in (0..num_args).zip(1..num_args) {
            result.append(&mut vec![
                "GET ".to_owned() + &i.to_string(),
                "GET ".to_owned() + &j.to_string(),
                mnemonic.to_owned(),
            ]);
            if i != 0 {
                result.push("AND".to_owned());
            }
        }
        for _ in 0..num_args {
            result.push("FALL".to_owned());
        }
    }
    result
}

fn lower_nary_fold_primitive<'a>(
    implementation_arity: usize,
    min_args: usize,
    default_argument: u64,
    mnemonic: &str,
    mut args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    stack_slots_used: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    assert!(
        args.len() >= min_args,
        "Too few arguments provided to nary fold primitive function."
    );
    while args.len() < implementation_arity {
        args.insert(0, Expression::Int(default_argument));
    }
    let mut stack_slots_used = stack_slots_used;
    for (i, arg) in args.into_iter().enumerate() {
        result.append(&mut lower_expression(arg, &env.clone(), stack_slots_used));
        stack_slots_used += 1; // arg
        if (i == implementation_arity - 1)
            || (i >= implementation_arity && ((i % (implementation_arity - 1)) == 0))
        {
            result.push(mnemonic.to_owned());
            // Note: this cannot be rewritten as
            // `stack_slots_used -= 1 - implementation_arity`
            // because that will promote 1 to usize, and then underflow.
            stack_slots_used -= implementation_arity; // implementation args
            stack_slots_used += 1; //                    implementation result
        }
    }
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
            b"let" => lower_let(args, env, stack_slots_used),
            b"if" => lower_if(args, env, stack_slots_used),
            b"add1" => lower_unary_primitive("ADD1", args, env, stack_slots_used),
            b"sub1" => lower_unary_primitive("SUB1", args, env, stack_slots_used),
            b"zero?" => lower_unary_primitive("ZEROP", args, env, stack_slots_used),
            b"integer?" => lower_unary_primitive("INTEGERP", args, env, stack_slots_used),
            b"boolean?" => lower_unary_primitive("BOOLEANP", args, env, stack_slots_used),
            b"char?" => lower_unary_primitive("CHARP", args, env, stack_slots_used),
            b"null?" => lower_unary_primitive("NULLP", args, env, stack_slots_used),
            b"not" => lower_unary_primitive("NOT", args, env, stack_slots_used),
            b"char->integer" => lower_unary_primitive("CHARTOINT", args, env, stack_slots_used),
            b"integer->char" => lower_unary_primitive("INTTOCHAR", args, env, stack_slots_used),
            b"+" => lower_nary_fold_primitive(2, 0, 0, "ADD", args, env, stack_slots_used),
            b"-" => lower_nary_fold_primitive(2, 1, 0, "SUB", args, env, stack_slots_used),
            b"*" => lower_nary_fold_primitive(2, 0, 1, "MUL", args, env, stack_slots_used),
            b"<" => lower_nary_all_pairs_primitive(2, "LT", args, env, stack_slots_used),
            b"=" => lower_nary_all_pairs_primitive(2, "EQ", args, env, stack_slots_used),
            b"eq?" => lower_nary_all_pairs_primitive(2, "EQP", args, env, stack_slots_used),
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
        Expression::Char(x) => vec!["LOAD #\\".to_owned() + format!("x{x:x}").as_str()],
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
    assert!(input_slice.is_empty(), "Leftover data: {input_slice:?}");
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
#[should_panic(expected = "Leftover data: [93]")]
fn leftover_data() {
    compile_all(b"]");
}

#[test]
#[should_panic(expected = "incorrect argument count for unary primitive function")]
fn too_few_unary_args() {
    compile_all(b"(not)");
}

#[test]
#[should_panic(expected = "incorrect argument count for unary primitive function")]
fn too_many_unary_args() {
    compile_all(b"(not 1 2)");
}

#[test]
#[should_panic(expected = "Too few arguments provided to nary fold primitive function")]
fn too_few_nary_args() {
    compile_all(b"(-)");
}

#[test]
#[should_panic(expected = "Couldn't find environment entry for \"a\"")]
fn use_undefined_variable() {
    compile_all(b"a");
}
