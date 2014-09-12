use std::fmt::{Show, Formatter, Result};

#[deriving(PartialOrd, PartialEq, Clone)]
pub enum Value {
    List(Vec<Value>),
    Symbol(String),
    String_(String),
    Number(f64),
}

impl Show for Value {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            List(ref vals) => {
                try!(write!(f, "("));
                for (i, val) in vals.iter().enumerate() {
                    try!(write!(f, "{}", val));
                    if i + 1 != vals.len() {
                        try!(write!(f, " "));
                    }
                }
                write!(f, ")")
            },
            Symbol(ref val) => val.fmt(f),
            String_(ref val) => write!(f, "\"{}\"", val),
            Number(ref val) => val.fmt(f),
        }
    }
}

#[deriving(Show, PartialEq)]
pub enum Token {
    LPAREN,
    RPAREN,
    NUM(f64),
    SYM(String),
    STR(String),
}

struct Lexer<R> {
    stream: std::iter::Peekable<char, R>,
}

fn is_ident(c: char) -> bool {
    c.is_alphabetic() || c == '-'
}

impl<R: Iterator<char>> Iterator<Token> for Lexer<R> {
    fn next(&mut self) -> Option<Token> {
        loop {
            match self.stream.next() {
                None => return None,
                Some(c) => {
                    match c {
                        '(' => return Some(LPAREN),
                        ')' => return Some(RPAREN),
                        '"' => {
                            let mut res = String::new();
                            while self.stream.peek().map_or(false, |&c| c != '"') {
                                res.push_char(self.stream.next().unwrap());
                            }
                            assert!(self.stream.next().unwrap() == '"');
                            return Some(STR(res));
                        },
                        c if is_ident(c) => {
                            let mut res = String::new();
                            res.push_char(c);
                            while self.stream.peek().map_or(false, |&c| c.is_alphabetic()) {
                                res.push_char(self.stream.next().unwrap());
                            }
                            return Some(SYM(res));
                        },
                        c @ '0' .. '9' | c @ '.' => {
                            let mut res = String::new();
                            if c != '.' {
                                res.push_char(c);
                                while self.stream.peek().map_or(false, |&c| c.is_digit()) {
                                    res.push_char(self.stream.next().unwrap());
                                }
                                if self.stream.peek().map_or(false, |&c| c == '.') {
                                    res.push_char(self.stream.next().unwrap());
                                    while self.stream.peek().map_or(false, |&c| c.is_digit()) {
                                        res.push_char(self.stream.next().unwrap());
                                    }
                                }
                            } else {
                                res.push_char('0');
                                res.push_char('.');
                                while self.stream.peek().map_or(false, |&c| c.is_digit()) {
                                    res.push_char(self.stream.next().unwrap());
                                }
                            }

                            return Some(NUM(from_str(res.as_slice()).unwrap()));
                        },
                        '#' => {
                            while self.stream.peek().map_or(false, |&c| c != '\n') {
                                self.stream.next();
                            }
                            continue;
                        },
                        ' ' | '\n' | '\t' => {
                            continue;
                        }
                        c => {
                            println!("Invalid character: {}", c);
                            return None;
                        }
                    }
                }
            }
        }
    }
}

struct Parser<R> {
    lexer: std::iter::Peekable<Token, Lexer<R>>,
    stack: Vec<Value>,
}

impl<R: Iterator<char>> Parser<R> {
    fn parse(&mut self) -> Option<Value> {
        match self.lexer.next() {
            None => return None,
            Some(tok) => {
                match tok {
                    NUM(val) => return Some(Number(val)),
                    SYM(val) => return Some(Symbol(val)),
                    STR(val) => return Some(String_(val)),
                    LPAREN => {
                        while self.lexer.peek().map_or(false, |tok| tok != &RPAREN) {
                            let next = self.parse().expect("Needed an element");
                            self.stack.push(next);
                        }
                        assert_eq!(self.lexer.next().unwrap(), RPAREN);
                        let mut st = Vec::new();
                        std::mem::swap(&mut self.stack, &mut st);
                        return Some(List(st));
                    },
                    RPAREN => {
                        println!("Unbalanced parenthesis!");
                        return None;
                    }
                }
            }
        }
    }
}

pub fn parse_str(s: &str) -> Value {
    parse(s.chars().peekable())
}

pub fn parse<R: Iterator<char>>(iter: std::iter::Peekable<char, R>) -> Value {
    let l = Lexer { stream: iter };
    let mut p = Parser { lexer: l.peekable(), stack: Vec::new() };
    p.parse().unwrap()
}
