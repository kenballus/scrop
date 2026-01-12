use std::fmt::{Display, Error, Formatter};
use std::io::{Read, stdin};

#[derive(Debug)]
enum Expression<'a> {
    Int(u64),
    Bool(bool),
    Char(u8),
    Symbol(&'a [u8]),
    FnCall(&'a [u8], Vec<Expression<'a>>),
    Null,
}

impl<'a> Expression<'a> {
    fn fmt_with_depth(&self, _f: &mut Formatter<'_>, depth: usize) -> String {
        const INDENT_SIZE: usize = 4;
        let s = match self {
            Expression::Int(x) => &x.to_string(),
            Expression::Bool(x) => {
                if *x {
                    "#t"
                } else {
                    "#f"
                }
            }
            Expression::Symbol(x) => str::from_utf8(x).expect("Invalid utf8 in symbol, somehow?"),
            Expression::FnCall(name, args) => {
                &("(\n".to_owned()
                    + &" ".repeat((depth + 1) * INDENT_SIZE)
                    + str::from_utf8(name).expect("Invalid utf8 in fn name, somehow?")
                    + "\n"
                    + &args
                        .iter()
                        .map(|arg| arg.fmt_with_depth(_f, depth + 1))
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "\n"
                    + &" ".repeat(depth * INDENT_SIZE)
                    + ")")
            }
            Expression::Char(x) => &("#\\".to_owned() + format!("x{:x}", x).as_str()),
            Expression::Null => "'()",
        };
        " ".repeat(depth * INDENT_SIZE) + s
    }
}

impl<'a> Display for Expression<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.fmt_with_depth(f, 0).fmt(f)
    }
}

fn check_for_delimiter(input: &[u8]) -> bool {
    input.is_empty() || input[0].is_ascii_whitespace() || input[0] == b')'
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

fn consume_fn_call<'a>(input: &'a [u8]) -> Option<((&'a [u8], Vec<Expression<'a>>), &'a [u8])> {
    if input.is_empty() || input[0] != b'(' {
        return None;
    }
    let rem0 = &input[1..];
    if let Some((name, rem1)) = consume_symbol(rem0) {
        let (args, rem2) = consume_expressions(rem1);
        let rem3 = consume_whitespace(rem2);
        if rem3.is_empty() || rem3[0] != b')' {
            None
        } else {
            Some(((name, args), &rem3[1..]))
        }
    } else {
        panic!("Invalid function name")
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
    } else if let Some(((name, args), rem)) = consume_fn_call(input) {
        Some((Expression::FnCall(name, args), rem))
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
            Expression::FnCall(name, args) => {
                for arg in args {
                    result.append(&mut lower(vec![arg]));
                }
                result.push(
                    (match name {
                        b"add1" => "ADD1",
                        b"sub1" => "SUB1",
                        b"+" => "ADD",
                        b"-" => "SUB",
                        b"*" => "MUL",
                        b"<" => "LT",
                        b"=" => "EQ",
                        b"zero?" => "ZEROP",
                        b"integer?" => "INTEGERP",
                        b"boolean?" => "BOOLEANP",
                        b"char?" => "CHARP",
                        b"null?" => "NULLP",
                        b"not" => "NOT",
                        b"char->integer" => "CHARTOINT",
                        b"integer->char" => "INTTOCHAR",
                        _ => todo!(),
                    })
                    .to_owned(),
                );
            }
            Expression::Null => result.push("LOAD64 NULL".to_owned()),
            Expression::Symbol(_) => todo!(),
        };
    }
    result
}

fn main() {
    let mut input = Vec::new();
    let _bytes_read = stdin().read_to_end(&mut input);
    let (ast, rem) = consume_expressions(&input[..]);
    // for node in &ast {
    //     println!("{}", node)
    // }
    if !rem.is_empty() {
        panic!("Leftover data: {:?}", rem);
    }
    println!("{}", lower(ast).join("\n"))
}
