use std::{
    collections::{HashMap, HashSet},
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

fn consume_string_literal(input: &[u8]) -> Option<(Vec<u8>, &[u8])> {
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
    if let Some(input) = consume_bytes(input, b"'") {
        if let Some(input) = consume_bytes(consume_whitespace(input), b"(") {
            if let Some(input) = consume_bytes(consume_whitespace(input), b")") {
                Some(input)
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
    } else if let Some((sym, input)) = consume_symbol(input) {
        Some((Expression::Symbol(sym), input))
    } else if let Some((args, input)) = consume_form(input) {
        Some((Expression::Form(args), input))
    } else if let Some((v, input)) = consume_string_literal(input) {
        Some((Expression::String(v), input))
    } else if let Some(input) = consume_null(input) {
        Some((Expression::Null, input))
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
    instructions_emitted: usize,
    is_tail: bool,
) -> Vec<String> {
    if let Expression::Form(bindings) = args.remove(0) {
        let mut binding_names = Vec::new();
        let mut binding_exps = Vec::new();
        for binding in bindings {
            if let Expression::Form(mut binding) = binding {
                assert!(
                    binding.len() == 2,
                    "let binding has incorrect argument count."
                );
                binding_exps.push(binding.pop().unwrap());
                binding_names.push(binding.pop().unwrap());
            } else {
                panic!("let binding is not a form")
            }
        }
        let mut lambda = vec![
            Expression::Symbol(b"lambda"),
            Expression::Form(binding_names),
        ];
        lambda.append(&mut args);
        lower_call(
            Expression::Form(lambda),
            binding_exps,
            env,
            instructions_emitted,
            is_tail,
        )
    } else {
        panic!("let bindings is not a form")
    }
}

fn scan_expression_for_free_variables<'a>(
    exp: &Expression<'a>,
    env: &HashMap<&'a [u8], usize>,
    parameters: &HashSet<&'a [u8]>,
) -> HashSet<&'a [u8]> {
    let mut result = HashSet::new();
    match exp {
        // No validation happens here, because it is handled later by codegen
        Expression::Form(args) => {
            if let Some(arg0) = args.first() {
                if let Expression::Symbol(x) = arg0 {
                    if env.contains_key(x) {
                        // this must be first to allow binding over lambda and let
                        result.extend(scan_expressions_for_free_variables(args, env, parameters));
                    } else {
                        match arg0 {
                            Expression::Symbol(b"lambda") => {
                                let mut new_parameters = parameters.clone();
                                if let Some(Expression::Form(inner_params)) = args.get(1) {
                                    for inner_param in inner_params {
                                        if let Expression::Symbol(inner_param_symbol) = inner_param
                                        {
                                            new_parameters.insert(inner_param_symbol);
                                        }
                                    }
                                }
                                result.extend(scan_expressions_for_free_variables(
                                    &args[2..],
                                    env,
                                    &new_parameters,
                                ));
                            }
                            Expression::Symbol(b"let") => {
                                let mut new_parameters = parameters.clone();
                                if let Some(Expression::Form(bindings)) = args.get(1) {
                                    for binding in bindings {
                                        if let Expression::Form(binding_vec) = binding
                                            && let [Expression::Symbol(k), v] =
                                                binding_vec.as_slice()
                                        {
                                            result.extend(scan_expression_for_free_variables(v, env, parameters /* not let*, so bindings can't use each other. */));
                                            new_parameters.insert(k);
                                        }
                                    }
                                }
                                // If any of the above checks fail, it's fine. codegen will catch it.
                                result.extend(scan_expressions_for_free_variables(
                                    &args[2..],
                                    env,
                                    &new_parameters,
                                ));
                            }
                            _ => result
                                .extend(scan_expressions_for_free_variables(args, env, parameters)),
                        }
                    }
                } else {
                    result.extend(scan_expressions_for_free_variables(args, env, parameters));
                }
            }
        }
        Expression::Symbol(name) => {
            if !parameters.contains(name) && env.contains_key(name) {
                result.insert(name);
            }
        }
        _ => {}
    }
    result
}

fn scan_expressions_for_free_variables<'a>(
    exps: &[Expression<'a>],
    env: &HashMap<&'a [u8], usize>,
    parameters: &HashSet<&'a [u8]>,
) -> HashSet<&'a [u8]> {
    let mut result = HashSet::new();
    for exp in exps {
        result.extend(scan_expression_for_free_variables(exp, env, parameters).drain());
    }
    result
}

fn lower_call<'a>(
    func: Expression<'a>,
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
    is_tail: bool,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut new_env = env.clone();
    if !is_tail {
        result.push("FRAME".to_owned());
        for v in new_env.values_mut() {
            *v += 1;
        }
    }

    let num_args = args.len();
    for arg in args.into_iter().rev() {
        result.append(&mut lower_expression(
            arg,
            &new_env,
            instructions_emitted + result.len(),
            false,
        ));
        for v in new_env.values_mut() {
            *v += 1;
        }
    }
    result.append(&mut lower_expression(
        func,
        &new_env,
        instructions_emitted + result.len(),
        false,
    ));
    for v in new_env.values_mut() {
        *v += 1;
    }
    result.push(format!("LOAD {num_args} // call arity").to_owned());
    if is_tail {
        result.push("TAILCALL".to_owned());
    } else {
        result.push("CALL".to_owned());
    }
    result
}

fn lower_lambda<'a>(
    mut args: Vec<Expression<'a>>,
    lambda_name: Option<&'a [u8]>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    if let Expression::Form(parameters) = args.remove(0) {
        let mut parameter_set = HashSet::new();
        let mut parameter_names = Vec::new();
        let arity = parameters.len();
        for parameter in parameters {
            if let Expression::Symbol(x) = parameter {
                assert!(parameter_set.insert(x), "Duplicate argument in lambda");
                parameter_names.push(x);
            } else {
                panic!("lambda parameter is not a symbol");
            }
        }
        let free_var_set =
            scan_expressions_for_free_variables(args.as_slice(), env, &parameter_set);
        let free_vars: Vec<_> = free_var_set.iter().collect();
        result.append(&mut lower_variadic_primitive(
            0,
            "VECTOR",
            free_vars.iter().map(|x| Expression::Symbol(x)).collect(),
            env,
            instructions_emitted + result.len(),
        ));
        result.push(format!("LOAD {arity} // lambda arity").to_owned());
        result.push(
            format!(
                "LAMBDA {}",
                instructions_emitted + result.len() + 1 /* for LAMBDA */ + 1 /* for JUMP */
            )
            .to_owned(),
        );
        let mut lambda_env = HashMap::new();
        // caller pushes args in reverse order
        for p in parameter_names.into_iter().rev() {
            for v in lambda_env.values_mut() {
                *v += 1;
            }
            lambda_env.insert(p, 0);
        }
        // CALL unpacks the freevar vector into the stack in reverse order
        for f in free_vars.into_iter().rev() {
            for v in lambda_env.values_mut() {
                *v += 1;
            }
            lambda_env.insert(f, 0);
        }
        for v in lambda_env.values_mut() {
            *v += 1; // for the lambda
        }
        if let Some(fn_name) = lambda_name {
            lambda_env.insert(fn_name, 0);
        }
        for v in lambda_env.values_mut() {
            *v += 1; // for lr
        }

        let mut lambda_body = lower_expressions(
            args,
            &lambda_env,
            instructions_emitted + result.len() + 1, /* for JUMP */
            true,
        );
        lambda_body.push("RETURN".to_owned());
        result.push(format!("JUMP {}", lambda_body.len()));
        result.append(&mut lambda_body);
        result
    } else {
        panic!("lambda parameter list is invalid")
    }
}

fn lower_lambdarec<'a>(
    mut args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
) -> Vec<String> {
    if let Expression::Symbol(fn_name) = args.remove(0) {
        lower_lambda(args, Some(fn_name), env, instructions_emitted)
    } else {
        panic!("Invalid lambdarec name")
    }
}

fn lower_begin<'a>(
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
    is_tail: bool,
) -> Vec<String> {
    if args.is_empty() {
        // Technically wrong; whether begin allows 0 args is context-dependent
        vec!["LOAD UNSPECIFIED".to_owned()]
    } else {
        lower_expressions(args, env, instructions_emitted, is_tail)
    }
}

fn lower_if<'a>(
    mut args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
    is_tail: bool,
) -> Vec<String> {
    let mut result = Vec::new();
    assert!(matches!(args.len(), 2 | 3), "Invalid argument count to if");
    // cond
    result.append(&mut lower_expression(
        args.remove(0),
        env,
        instructions_emitted + result.len(),
        false,
    ));
    result.push("LOAD #f".to_owned());
    result.push("LOAD 2".to_owned());
    result.push("EQP ".to_owned());

    // consequent
    let mut consequent_code = lower_expression(
        args.remove(0),
        env,
        instructions_emitted + result.len() + 1, /* for CJUMP */
        is_tail,
    );

    // alternative
    let mut alternative_code = if let Some(alternative_code) = args.pop() {
        lower_expression(
            alternative_code,
            env,
            instructions_emitted + result.len() + 1 /* for CJUMP */ + consequent_code.len() + 1, /* for JUMP */
            is_tail,
        )
    } else {
        vec!["LOAD UNSPECIFIED".to_owned()]
    };

    consequent_code.push("JUMP ".to_owned() + &alternative_code.len().to_string());

    result.push("CJUMP ".to_owned() + &consequent_code.len().to_string());
    result.append(&mut consequent_code);
    result.append(&mut alternative_code);
    result
}

fn lower_list<'a>(
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
) -> Vec<String> {
    let mut new_env = env.clone();
    let mut result = Vec::new();
    let num_args = args.len();
    for arg in args {
        result.append(&mut lower_expression(
            arg,
            &new_env,
            instructions_emitted + result.len(),
            false,
        ));
        for v in new_env.values_mut() {
            *v += 1;
        }
    }
    result.push("LOAD NULL".to_owned());
    for _ in 0..num_args {
        result.push("CONS".to_owned());
    }
    result
}

fn lower_nary_primitive<'a>(
    mnemonic: &str,
    n: usize,
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    assert!(
        args.len() == n,
        "incorrect argument count for {n}-ary primitive"
    );
    let mut new_env = env.clone();
    for arg in args {
        result.append(&mut lower_expression(
            arg,
            &new_env,
            instructions_emitted + result.len(),
            false,
        ));
        for v in new_env.values_mut() {
            *v += 1;
        }
    }
    result.push(mnemonic.to_owned());
    result
}

fn lower_variadic_primitive<'a>(
    min_args: usize,
    mnemonic: &str,
    args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut new_env = env.clone();
    let num_args = args.len();
    assert!(
        num_args >= min_args,
        "Too few arguments provided to variadic primitive"
    );
    for arg in args.into_iter().rev() {
        result.append(&mut lower_expression(
            arg,
            &new_env,
            instructions_emitted + result.len(),
            false,
        ));
        for v in new_env.values_mut() {
            *v += 1;
        }
    }
    result.push(format!("LOAD {num_args} // primitive arity"));
    result.push(mnemonic.to_string());
    result
}

fn lower_form<'a>(
    mut args: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
    is_tail: bool,
) -> Vec<String> {
    assert!(!args.is_empty(), "Empty form!");
    let arg_0 = args.remove(0);
    if let Expression::Symbol(name) = arg_0 {
        if env.contains_key(name) {
            lower_call(
                Expression::Symbol(name),
                args,
                env,
                instructions_emitted,
                is_tail,
            )
        } else {
            match name {
                b"begin" => lower_begin(args, env, instructions_emitted, is_tail),
                b"lambdarec" => lower_lambdarec(args, env, instructions_emitted),
                b"let" => lower_let(args, env, instructions_emitted, is_tail),
                b"if" => lower_if(args, env, instructions_emitted, is_tail),
                b"list" => lower_list(args, env, instructions_emitted),
                b"lambda" => lower_lambda(args, None, env, instructions_emitted),
                b"add1" => lower_nary_primitive("ADD1", 1, args, env, instructions_emitted),
                b"sub1" => lower_nary_primitive("SUB1", 1, args, env, instructions_emitted),
                b"zero?" => lower_nary_primitive("ZEROP", 1, args, env, instructions_emitted),
                b"integer?" => lower_nary_primitive("INTEGERP", 1, args, env, instructions_emitted),
                b"boolean?" => lower_nary_primitive("BOOLEANP", 1, args, env, instructions_emitted),
                b"char?" => lower_nary_primitive("CHARP", 1, args, env, instructions_emitted),
                b"null?" => lower_nary_primitive("NULLP", 1, args, env, instructions_emitted),
                b"not" => lower_nary_primitive("NOT", 1, args, env, instructions_emitted),
                b"char->integer" => {
                    lower_nary_primitive("CHARTOINT", 1, args, env, instructions_emitted)
                }
                b"integer->char" => {
                    lower_nary_primitive("INTTOCHAR", 1, args, env, instructions_emitted)
                }
                b"+" => lower_variadic_primitive(0, "ADD", args, env, instructions_emitted),
                b"-" => lower_variadic_primitive(1, "SUB", args, env, instructions_emitted),
                b"*" => lower_variadic_primitive(0, "MUL", args, env, instructions_emitted),
                b"<" => lower_variadic_primitive(0, "LT", args, env, instructions_emitted),
                b"=" => lower_variadic_primitive(0, "EQ", args, env, instructions_emitted),
                b"eq?" => lower_variadic_primitive(0, "EQP", args, env, instructions_emitted),
                b"string" => lower_variadic_primitive(0, "STRING", args, env, instructions_emitted),
                b"string-append" => {
                    lower_variadic_primitive(0, "STRINGAPPEND", args, env, instructions_emitted)
                }
                b"string-ref" => {
                    lower_nary_primitive("STRINGREF", 2, args, env, instructions_emitted)
                }
                b"string-set!" => {
                    lower_nary_primitive("STRINGSET", 3, args, env, instructions_emitted)
                }
                b"vector" => lower_variadic_primitive(0, "VECTOR", args, env, instructions_emitted),
                b"vector-append" => {
                    lower_variadic_primitive(0, "VECTORAPPEND", args, env, instructions_emitted)
                }
                b"vector-ref" => {
                    lower_nary_primitive("VECTORREF", 2, args, env, instructions_emitted)
                }
                b"vector-set!" => {
                    lower_nary_primitive("VECTORSET", 3, args, env, instructions_emitted)
                }
                b"cons" => lower_nary_primitive("CONS", 2, args, env, instructions_emitted),
                b"car" => lower_nary_primitive("CAR", 1, args, env, instructions_emitted),
                b"cdr" => lower_nary_primitive("CDR", 1, args, env, instructions_emitted),
                _ => panic!("Cannot resolve symbol '{name:?}'"),
            }
        }
    } else {
        lower_call(arg_0, args, env, instructions_emitted, is_tail)
    }
}

fn lower_expression<'a>(
    exp: Expression<'a>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
    is_tail: bool,
) -> Vec<String> {
    match exp {
        Expression::Int(x) => vec!["LOAD ".to_owned() + &x.to_string()],
        Expression::Char(x) => vec![format!("LOAD #\\x{x:x}")],
        Expression::Bool(x) => vec!["LOAD ".to_owned() + if x { "#t" } else { "#f" }],
        Expression::Form(args) => lower_form(args, env, instructions_emitted, is_tail),
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
            instructions_emitted,
        ),
    }
}

fn lower_expressions<'a>(
    exps: Vec<Expression<'a>>,
    env: &HashMap<&'a [u8], usize>,
    instructions_emitted: usize,
    is_tail: bool,
) -> Vec<String> {
    if exps.is_empty() {
        vec!["LOAD UNSPECIFIED".to_owned()]
    } else {
        let mut result = Vec::new();
        let num_exps = exps.len();
        for (i, exp) in exps.into_iter().enumerate() {
            let is_last = i == num_exps - 1;
            result.append(&mut lower_expression(
                exp,
                env,
                instructions_emitted + result.len(),
                is_tail && is_last,
            ));
            if !is_last {
                result.push("FORGET".to_owned());
            }
        }
        result
    }
}

fn parse(input_slice: &[u8]) -> Vec<Expression<'_>> {
    let (ast, input_slice) = consume_expressions(consume_whitespace(input_slice));
    assert!(
        input_slice.is_empty(),
        "Parsing failed. Leftover data: {input_slice:?}"
    );
    ast
}

fn codegen(ast: Vec<Expression>) -> Vec<String> {
    lower_expressions(ast, &HashMap::new(), 0, false)
}

fn main() {
    let mut input_vec = Vec::new();
    input_vec.extend_from_slice(b"((lambda () ");
    let _bytes_read = stdin().read_to_end(&mut input_vec);
    input_vec.extend_from_slice(b"))");
    let mut user_code = codegen(parse(input_vec.as_slice()));
    user_code.push("DONE".to_owned());
    println!("{}", user_code.join("\n"));
}

#[test]
#[should_panic(expected = "let bindings is not a form")]
fn invalid_let_binding_list() {
    codegen(parse(b"(let 1 1)"));
}

#[test]
#[should_panic(expected = "let binding is not a form")]
fn invalid_let_binding_list_entry() {
    codegen(parse(b"(let (1) 1)"));
}

#[test]
#[should_panic(expected = "let binding has incorrect argument count.")]
fn let_binding_too_many_args() {
    codegen(parse(b"(let ((x 1 1)) x)"));
}

#[test]
#[should_panic(expected = "Duplicate argument in lambda")]
fn let_binding_duplicate_key() {
    codegen(parse(b"(let ((x 1) (x 1)) x)"));
}

#[test]
#[should_panic(expected = "let binding is not a form")]
fn let_binding_list_not_nested() {
    codegen(parse(b"(let (x 1) x)"));
}

#[test]
#[should_panic(expected = "Invalid argument count to if")]
fn too_few_if_args() {
    codegen(parse(b"(if)"));
}

#[test]
#[should_panic(expected = "Invalid argument count to if")]
fn too_many_if_args() {
    codegen(parse(b"(if 1 2 3 4)"));
}

#[test]
#[should_panic(expected = "Parsing failed. Leftover data: [93]")]
fn leftover_data() {
    codegen(parse(b"]"));
}

#[test]
#[should_panic(expected = "incorrect argument count for 1-ary primitive")]
fn too_few_unary_args() {
    codegen(parse(b"(not)"));
}

#[test]
#[should_panic(expected = "incorrect argument count for 1-ary primitive")]
fn too_many_unary_args() {
    codegen(parse(b"(not 1 2)"));
}

#[test]
#[should_panic(expected = "Too few arguments provided to variadic primitive")]
fn too_few_variadic_args() {
    codegen(parse(b"(-)"));
}

#[test]
#[should_panic(expected = "Couldn't find environment entry for \"a\"")]
fn use_undefined_variable() {
    codegen(parse(b"a"));
}

#[test]
#[should_panic(expected = "Parsing failed. Leftover data: [35, 124, 32, 35, 124, 32, 124, 35]")]
fn mismatched_nested_comment() {
    codegen(parse(b"#| #| |#"));
}

#[test]
#[should_panic(expected = "lambda parameter is not a symbol")]
fn numeric_symbol() {
    codegen(parse(b"(let ((1 0)) 1)"));
}
