//! Motor de expressões da calculadora: normalização, tokenização, parser
//! recursivo-descendente e avaliador. Tudo puro (sem I/O e sem dependências),
//! portanto fácil de testar — veja `tests/eval.rs`.
//!
//! Precedência (da mais fraca para a mais forte):
//! 1. `+` `-`
//! 2. `*` `/` `%` (resto), com multiplicação implícita (`2π`, `2(3)`)
//! 3. unário `-` `+`
//! 4. `^` (associativo à direita; `2^3^2 = 512`)
//! 5. pós-fixo `!` (fatorial) e `%` (porcentagem: `50% = 0,5`)
//! 6. fator (número, constante, função, parênteses)

use std::fmt;

pub const PI: f64 = std::f64::consts::PI;
pub const E: f64 = std::f64::consts::E;

/// Modo angular usado pelas funções trigonométricas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleMode {
    /// Ângulos interpretados em radianos (padrão).
    Radians,
    /// Ângulos interpretados em graus.
    Degrees,
}

/// Opções de avaliação de uma expressão.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub angle: AngleMode,
    /// Último resultado calculado, disponível na expressão como `ans`.
    pub ans: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            angle: AngleMode::Radians,
            ans: 0.0,
        }
    }
}

/// Erros de avaliação com mensagens em português.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// Expressão vazia.
    Empty,
    /// Parêntese sem par.
    Unbalanced(String),
    /// Token que não faz sentido na posição atual.
    Unexpected(String),
    /// Nome de função/constante desconhecido.
    UnknownName(String),
    /// Número errado de argumentos para uma função.
    ArgCount(String),
    /// Divisão/resto por zero.
    DivisionByZero,
    /// Resultado fora do domínio ou do intervalo suportado.
    Domain(String),
    /// Fatorial pedido para valor não suportado.
    Factorial(f64),
    /// Faltou um valor (por exemplo, `2+`).
    NeedValue,
    /// Resultado infinito (fora do intervalo de `f64`).
    Overflow,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Expressão vazia"),
            Self::Unbalanced(c) => write!(f, "Parênteses desbalanceados ({c})"),
            Self::Unexpected(t) => write!(f, "Token inesperado: {t}"),
            Self::UnknownName(n) => write!(f, "Função ou constante desconhecida: {n}"),
            Self::ArgCount(n) => write!(f, "Número de argumentos inválido para {n}"),
            Self::DivisionByZero => write!(f, "Divisão por zero"),
            Self::Domain(m) => write!(f, "Domínio inválido: {m}"),
            Self::Factorial(n) => {
                write!(f, "Fatorial inválido para {n}; use um inteiro ≥ 0 e ≤ 170")
            }
            Self::NeedValue => write!(f, "Valor esperado"),
            Self::Overflow => write!(f, "Resultado fora do intervalo suportado"),
        }
    }
}

impl std::error::Error for EvalError {}

/// Avalia uma expressão matemática e retorna o resultado.
///
/// Aceita números (`12`, `.5`, `1e3`), operadores, constantes (`pi`, `e`,
/// `tau`, `ans`), funções e caracteres tipográficos (`π × ÷ − ² ³ √`).
pub fn evaluate(input: &str, opts: Options) -> Result<f64, EvalError> {
    let cleaned = balance(&normalize(input));
    if cleaned.trim().is_empty() {
        return Err(EvalError::Empty);
    }
    let toks = tokenize(&cleaned)?;
    let mut parser = Parser { toks, pos: 0 };
    let value = parser.parse(opts)?;
    if value.is_nan() {
        return Err(EvalError::Domain("resultado indefinido".to_string()));
    }
    if value.is_infinite() {
        return Err(EvalError::Overflow);
    }
    Ok(value)
}

/// Converte caracteres tipográficos para a sintaxe textual da engine.
fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'π' => "pi".to_string(),
            '×' => "*".to_string(),
            '÷' => "/".to_string(),
            '−' | '–' | '—' => "-".to_string(),
            '²' => "^2".to_string(),
            '³' => "^3".to_string(),
            '√' => "sqrt(".to_string(),
            other => other.to_string(),
        })
        .collect()
}

/// Fecha automaticamente parênteses abertos no fim da expressão.
/// É o que permite digitar `sqrt(9` e receber o resultado sem o `)` final.
fn balance(s: &str) -> String {
    let mut open = 0i32;
    for c in s.chars() {
        if c == '(' {
            open += 1;
        } else if c == ')' {
            open -= 1;
        }
    }
    let mut out = s.to_string();
    for _ in 0..open.max(0) {
        out.push(')');
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    /// `true` = porcentagem pós-fixa (`50%` ⇒ `0,5`); `false` = resto binário (`8%3`).
    Percent(bool),
    Caret,
    Bang,
    LParen,
    RParen,
    Comma,
}

fn tokenize(s: &str) -> Result<Vec<Tok>, EvalError> {
    let chars: Vec<char> = s.chars().collect();
    let mut toks: Vec<Tok> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            c if c.is_whitespace() => i += 1,
            '0'..='9' | '.' => {
                let (num, next) = parse_number(&chars, i)?;
                toks.push(Tok::Num(num));
                i = next;
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                toks.push(Tok::Ident(chars[start..i].iter().collect()));
            }
            '+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                toks.push(Tok::Star);
                i += 1;
            }
            '/' => {
                toks.push(Tok::Slash);
                i += 1;
            }
            '^' => {
                toks.push(Tok::Caret);
                i += 1;
            }
            '!' => {
                toks.push(Tok::Bang);
                i += 1;
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            ',' => {
                toks.push(Tok::Comma);
                i += 1;
            }
            '%' => {
                let next = skip_space(&chars, i + 1)
                    .and_then(|j| chars.get(j).copied())
                    .map(|c| {
                        c.is_ascii_digit()
                            || c == '.'
                            || c == '('
                            || c.is_ascii_alphabetic()
                            || c == '_'
                    })
                    .unwrap_or(false);
                let prev_is_value = toks
                    .last()
                    .map(|t| matches!(t, Tok::Num(_) | Tok::Ident(_) | Tok::RParen))
                    .unwrap_or(false);
                toks.push(Tok::Percent(prev_is_value && !next));
                i += 1;
            }
            other => return Err(EvalError::Unexpected(format!("caractere '{other}'"))),
        }
    }
    Ok(insert_implicit(toks))
}

fn skip_space(chars: &[char], mut i: usize) -> Option<usize> {
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    (i < chars.len()).then_some(i)
}

/// Lê um número (mantissa + ponto decimal + expoente `e`/`E`), devolvendo
/// o valor e a posição seguinte. O expoente só é consumido quando há dígitos
/// depois, então `2e` continua sendo "2" seguido da constante `e`.
fn parse_number(chars: &[char], start: usize) -> Result<(f64, usize), EvalError> {
    let mut i = start;
    let mut dots = 0usize;
    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
        if chars[i] == '.' {
            dots += 1;
        }
        i += 1;
    }
    let mut end = i;
    if end < chars.len() && (chars[end] == 'e' || chars[end] == 'E') {
        let mut k = end + 1;
        if k < chars.len() && (chars[k] == '+' || chars[k] == '-') {
            k += 1;
        }
        if k < chars.len() && chars[k].is_ascii_digit() {
            while k < chars.len() && chars[k].is_ascii_digit() {
                k += 1;
            }
            end = k;
        }
    }
    let text: String = chars[start..end].iter().collect();
    if text == "." {
        return Err(EvalError::Unexpected("'.'".to_string()));
    }
    if dots > 1 {
        return Err(EvalError::Unexpected(format!("número '{text}'")));
    }
    let value = text
        .parse::<f64>()
        .map_err(|_| EvalError::Unexpected(format!("número '{text}'")))?;
    Ok((value, end))
}

/// Insere `*` em multiplicações implícitas: `2π`, `2(3)`, `(1+2)(3)`.
/// `sin(2)` não recebe o `*` (é chamada de função).
fn insert_implicit(toks: Vec<Tok>) -> Vec<Tok> {
    let mut out = Vec::with_capacity(toks.len() * 2);
    for t in toks {
        if let Some(last) = out.last() {
            let lhs_ends_value = matches!(last, Tok::Num(_) | Tok::Ident(_) | Tok::RParen);
            let rhs_starts_value = matches!(t, Tok::Num(_) | Tok::Ident(_) | Tok::LParen);
            let is_call = matches!(last, Tok::Ident(_)) && matches!(t, Tok::LParen);
            if lhs_ends_value && rhs_starts_value && !is_call {
                out.push(Tok::Star);
            }
        }
        out.push(t);
    }
    out
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn advance(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse(&mut self, opts: Options) -> Result<f64, EvalError> {
        let value = self.add_sub(opts)?;
        if let Some(t) = self.peek() {
            return Err(EvalError::Unexpected(tok_desc(t)));
        }
        Ok(value)
    }

    fn add_sub(&mut self, opts: Options) -> Result<f64, EvalError> {
        let mut v = self.mul_div(opts)?;
        loop {
            match self.peek() {
                Some(Tok::Plus) => {
                    self.advance();
                    v += self.mul_div(opts)?;
                }
                Some(Tok::Minus) => {
                    self.advance();
                    v -= self.mul_div(opts)?;
                }
                _ => break,
            }
        }
        Ok(v)
    }

    fn mul_div(&mut self, opts: Options) -> Result<f64, EvalError> {
        let mut v = self.unary(opts)?;
        loop {
            match self.peek() {
                Some(Tok::Star) => {
                    self.advance();
                    v *= self.unary(opts)?;
                }
                Some(Tok::Slash) => {
                    self.advance();
                    let r = self.unary(opts)?;
                    if r == 0.0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    v /= r;
                }
                Some(Tok::Percent(false)) => {
                    self.advance();
                    let r = self.unary(opts)?;
                    if r == 0.0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    v %= r;
                }
                _ => break,
            }
        }
        Ok(v)
    }

    fn unary(&mut self, opts: Options) -> Result<f64, EvalError> {
        match self.peek() {
            Some(Tok::Plus) => {
                self.advance();
                self.unary(opts)
            }
            Some(Tok::Minus) => {
                self.advance();
                Ok(-self.unary(opts)?)
            }
            _ => self.power(opts),
        }
    }

    /// `^` associa à direita e toma um operando unário à direita, então
    /// `2^-3` funciona e `-2^2 = -(2^2) = -4`.
    fn power(&mut self, opts: Options) -> Result<f64, EvalError> {
        let base = self.postfix(opts)?;
        if matches!(self.peek(), Some(Tok::Caret)) {
            self.advance();
            let exponent = self.unary(opts)?;
            return Ok(base.powf(exponent));
        }
        Ok(base)
    }

    fn postfix(&mut self, opts: Options) -> Result<f64, EvalError> {
        let mut v = self.primary(opts)?;
        loop {
            match self.peek() {
                Some(Tok::Bang) => {
                    self.advance();
                    v = factorial(v)?;
                }
                Some(Tok::Percent(true)) => {
                    self.advance();
                    v /= 100.0;
                }
                _ => break,
            }
        }
        Ok(v)
    }

    fn primary(&mut self, opts: Options) -> Result<f64, EvalError> {
        match self.advance() {
            None => Err(EvalError::NeedValue),
            Some(Tok::Num(n)) => Ok(n),
            Some(Tok::LParen) => {
                let v = self.add_sub(opts)?;
                match self.advance() {
                    Some(Tok::RParen) => Ok(v),
                    _ => Err(EvalError::Unbalanced("faltando ')'".to_string())),
                }
            }
            Some(Tok::Ident(name)) => self.ident_value(&name, opts),
            Some(other) => Err(EvalError::Unexpected(tok_desc(&other))),
        }
    }

    fn ident_value(&mut self, name: &str, opts: Options) -> Result<f64, EvalError> {
        match name {
            "pi" => return Ok(PI),
            "tau" => return Ok(std::f64::consts::TAU),
            "e" => return Ok(E),
            "ans" => return Ok(opts.ans),
            _ => {}
        }
        match self.peek() {
            Some(Tok::LParen) => {
                self.advance();
                let mut args: Vec<f64> = Vec::new();
                if !matches!(self.peek(), Some(Tok::RParen)) {
                    loop {
                        args.push(self.add_sub(opts)?);
                        match self.peek() {
                            Some(Tok::Comma) => {
                                self.advance();
                            }
                            Some(Tok::RParen) => break,
                            _ => return Err(EvalError::Unbalanced("faltando ')'".to_string())),
                        }
                    }
                }
                self.advance(); // ')'
                call(name, &args, opts.angle)
            }
            _ => Err(EvalError::UnknownName(name.to_string())),
        }
    }
}

fn tok_desc(t: &Tok) -> String {
    match t {
        Tok::Num(n) => format!("número {n}"),
        Tok::Ident(s) => s.clone(),
        Tok::Plus => "'+'".to_string(),
        Tok::Minus => "'−'".to_string(),
        Tok::Star => "'×'".to_string(),
        Tok::Slash => "'÷'".to_string(),
        Tok::Percent(_) => "'%'".to_string(),
        Tok::Caret => "'^'".to_string(),
        Tok::Bang => "'!'".to_string(),
        Tok::LParen => "'('".to_string(),
        Tok::RParen => "')'".to_string(),
        Tok::Comma => "','".to_string(),
    }
}

fn factorial(v: f64) -> Result<f64, EvalError> {
    if v < 0.0 || v != v.trunc() {
        return Err(EvalError::Factorial(v));
    }
    if v > 170.0 {
        return Err(EvalError::Overflow);
    }
    let mut acc: f64 = 1.0;
    let n = v as u64;
    for i in 2..=n {
        acc *= i as f64;
    }
    Ok(acc)
}

macro_rules! one {
    ($n:expr, $name:expr, $e:expr) => {{
        if $n != 1 {
            return Err(EvalError::ArgCount($name.to_owned()));
        }
        Ok($e)
    }};
}

macro_rules! two {
    ($n:expr, $name:expr, $e:expr) => {{
        if $n != 2 {
            return Err(EvalError::ArgCount($name.to_owned()));
        }
        Ok($e)
    }};
}

fn call(name: &str, args: &[f64], angle: AngleMode) -> Result<f64, EvalError> {
    let n = args.len();
    let deg = angle == AngleMode::Degrees;
    match name {
        "sin" | "cos" | "tan" => one!(n, name, {
            let x = if deg { args[0].to_radians() } else { args[0] };
            match name {
                "sin" => x.sin(),
                "cos" => x.cos(),
                _ => x.tan(),
            }
        }),
        "asin" | "acos" | "atan" => one!(n, name, {
            let x = args[0];
            let r = match name {
                "asin" => x.asin(),
                "acos" => x.acos(),
                _ => x.atan(),
            };
            if r.is_nan() {
                return Err(EvalError::Domain(format!(
                    "{name} exige valor entre -1 e 1 (recebeu {x})"
                )));
            }
            if deg {
                r.to_degrees()
            } else {
                r
            }
        }),
        "sinh" => one!(n, name, args[0].sinh()),
        "cosh" => one!(n, name, args[0].cosh()),
        "tanh" => one!(n, name, args[0].tanh()),
        "asinh" => one!(n, name, args[0].asinh()),
        "acosh" => one!(n, name, args[0].acosh()),
        "atanh" => one!(n, name, args[0].atanh()),
        "ln" => one!(n, name, {
            if args[0] <= 0.0 {
                return Err(EvalError::Domain(
                    "ln exige argumento maior que zero".to_string(),
                ));
            }
            args[0].ln()
        }),
        "log" => one!(n, name, {
            if args[0] <= 0.0 {
                return Err(EvalError::Domain(
                    "log exige argumento maior que zero".to_string(),
                ));
            }
            args[0].log10()
        }),
        "log2" => one!(n, name, {
            if args[0] <= 0.0 {
                return Err(EvalError::Domain(
                    "log2 exige argumento maior que zero".to_string(),
                ));
            }
            args[0].log2()
        }),
        "sqrt" => one!(n, name, {
            if args[0] < 0.0 {
                return Err(EvalError::Domain(
                    "raiz quadrada de número negativo".to_string(),
                ));
            }
            args[0].sqrt()
        }),
        "cbrt" => one!(n, name, args[0].cbrt()),
        "abs" => one!(n, name, args[0].abs()),
        "floor" => one!(n, name, args[0].floor()),
        "ceil" => one!(n, name, args[0].ceil()),
        "round" => one!(n, name, args[0].round()),
        "sign" => one!(
            n,
            name,
            ((args[0] > 0.0) as i8 as f64) - ((args[0] < 0.0) as i8 as f64)
        ),
        "exp" => one!(n, name, args[0].exp()),
        "pow" => two!(n, name, args[0].powf(args[1])),
        "min" => two!(n, name, args[0].min(args[1])),
        "max" => two!(n, name, args[0].max(args[1])),
        "hypot" => two!(n, name, args[0].hypot(args[1])),
        _ => Err(EvalError::UnknownName(name.to_string())),
    }
}

/// Formata um valor para exibição, cortando zeros e sem notação
/// científica para números "normais".
pub fn format_value(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    let abs = v.abs();
    if v == v.trunc() && abs < 1e15 {
        return format!("{}", v as i64);
    }
    if abs >= 1e12 || abs < 1e-9 {
        return trim_exp(format!("{:e}", v));
    }
    let s = format!("{:.10}", v);
    let t = s.trim_end_matches('0').trim_end_matches('.');
    if t == "-0" {
        "0".to_string()
    } else {
        t.to_string()
    }
}

/// Tira zeros do fim da mantissa em notação científica (`1.500e3` → `1.5e3`).
fn trim_exp(s: String) -> String {
    if let Some((mant, exp)) = s.split_once('e') {
        let m = mant.trim_end_matches('0').trim_end_matches('.');
        format!("{m}e{exp}")
    } else {
        s
    }
}
