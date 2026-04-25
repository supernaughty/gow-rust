use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::fs::File;

use anyhow::{anyhow, Result};
use clap::Parser as ClapParser;
use regex::Regex;
use thiserror::Error;

// ============================================================
// CLI Args
// ============================================================

#[derive(ClapParser, Debug)]
#[command(name = "awk", about = "Pattern scanning and processing language")]
struct Args {
    /// Field separator (default: whitespace)
    #[arg(short = 'F', long = "field-separator", default_value = " ")]
    field_sep: String,

    /// Assign a value to a variable before program executes
    #[arg(short = 'v', long = "assign", value_name = "VAR=VALUE")]
    assign: Vec<String>,

    /// Read AWK program from file
    #[arg(short = 'f', long = "file")]
    program_file: Option<PathBuf>,

    /// AWK program text (required unless -f is used)
    #[arg(value_name = "PROGRAM")]
    program: Option<String>,

    /// Input files
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
}

// ============================================================
// Errors
// ============================================================

#[derive(Error, Debug)]
enum AwkError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("invalid -v assignment: {0}")]
    InvalidAssignment(String),
}

// ============================================================
// Lexer
// ============================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Literals
    Number(f64),
    Str(String),
    Regex(String),
    // Identifiers / keywords
    Ident(String),
    // Keywords
    Begin,
    End,
    Print,
    Printf,
    If,
    Else,
    While,
    Do,
    For,
    In,
    Delete,
    Next,
    Exit,
    Return,
    Function,
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Bang,
    Tilde,
    NotTilde,
    Dollar,
    PlusPlus,
    MinusMinus,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    Assign,
    Append,
    // Punctuation
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Semicolon,
    Comma,
    Newline,
    Pipe,
    // Field access
    Eof,
}

fn lex(src: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;

    // Track whether we expect a regex (after certain tokens)
    let can_be_regex = |tokens: &[Token]| -> bool {
        match tokens.last() {
            None => true,
            Some(t) => matches!(
                t,
                Token::Assign
                    | Token::PlusEq
                    | Token::MinusEq
                    | Token::StarEq
                    | Token::SlashEq
                    | Token::PercentEq
                    | Token::LBrace
                    | Token::LParen
                    | Token::Comma
                    | Token::Semicolon
                    | Token::Newline
                    | Token::Bang
                    | Token::Tilde
                    | Token::NotTilde
                    | Token::And
                    | Token::Or
                    | Token::In
                    | Token::If
                    | Token::While
                    | Token::For
                    | Token::Do
                    | Token::Else
                    | Token::Print
                    | Token::Printf
            ),
        }
    };

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace (but not newlines — they can be statement separators)
        if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
            continue;
        }

        // Newline
        if c == '\n' {
            // Only emit if meaningful (after a statement-ending token)
            match tokens.last() {
                Some(
                    Token::RBrace
                    | Token::RParen
                    | Token::RBracket
                    | Token::Ident(_)
                    | Token::Number(_)
                    | Token::Str(_)
                    | Token::Regex(_)
                    | Token::PlusPlus
                    | Token::MinusMinus
                    | Token::Next
                    | Token::Exit
                    | Token::Return
                    | Token::Delete,
                ) => tokens.push(Token::Newline),
                _ => {}
            }
            i += 1;
            continue;
        }

        // Line comment
        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // String literal
        if c == '"' {
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        'a' => s.push('\x07'),
                        'b' => s.push('\x08'),
                        'f' => s.push('\x0C'),
                        'v' => s.push('\x0B'),
                        '/' => s.push('/'),
                        other => {
                            s.push('\\');
                            s.push(other);
                        }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1; // consume closing "
            }
            tokens.push(Token::Str(s));
            continue;
        }

        // Regex literal — only when contextually appropriate
        if c == '/' && can_be_regex(&tokens) {
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '/' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        '/' => s.push('/'),
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        other => {
                            s.push('\\');
                            s.push(other);
                        }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1; // consume closing /
            }
            tokens.push(Token::Regex(s));
            continue;
        }

        // Numbers
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            // Scientific notation
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let num_str: String = chars[start..i].iter().collect();
            let n: f64 = num_str
                .parse()
                .map_err(|_| AwkError::Parse(format!("invalid number: {}", num_str)))?;
            tokens.push(Token::Number(n));
            continue;
        }

        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let tok = match word.as_str() {
                "BEGIN" => Token::Begin,
                "END" => Token::End,
                "print" => Token::Print,
                "printf" => Token::Printf,
                "if" => Token::If,
                "else" => Token::Else,
                "while" => Token::While,
                "do" => Token::Do,
                "for" => Token::For,
                "in" => Token::In,
                "delete" => Token::Delete,
                "next" => Token::Next,
                "exit" => Token::Exit,
                "return" => Token::Return,
                "function" => Token::Function,
                _ => Token::Ident(word),
            };
            tokens.push(tok);
            continue;
        }

        // Operators
        match c {
            '+' => {
                if i + 1 < chars.len() && chars[i + 1] == '+' {
                    tokens.push(Token::PlusPlus);
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::PlusEq);
                    i += 2;
                } else {
                    tokens.push(Token::Plus);
                    i += 1;
                }
            }
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '-' {
                    tokens.push(Token::MinusMinus);
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::MinusEq);
                    i += 2;
                } else {
                    tokens.push(Token::Minus);
                    i += 1;
                }
            }
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::StarEq);
                    i += 2;
                } else {
                    tokens.push(Token::Star);
                    i += 1;
                }
            }
            '/' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::SlashEq);
                    i += 2;
                } else {
                    tokens.push(Token::Slash);
                    i += 1;
                }
            }
            '%' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::PercentEq);
                    i += 2;
                } else {
                    tokens.push(Token::Percent);
                    i += 1;
                }
            }
            '^' => {
                tokens.push(Token::Caret);
                i += 1;
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Eq);
                    i += 2;
                } else {
                    tokens.push(Token::Assign);
                    i += 1;
                }
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ne);
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1] == '~' {
                    tokens.push(Token::NotTilde);
                    i += 2;
                } else {
                    tokens.push(Token::Bang);
                    i += 1;
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Le);
                    i += 2;
                } else {
                    tokens.push(Token::Lt);
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ge);
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::Append);
                    i += 2;
                } else {
                    tokens.push(Token::Gt);
                    i += 1;
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token::And);
                    i += 2;
                } else {
                    return Err(AwkError::Parse(format!("unexpected char: {}", c)).into());
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    tokens.push(Token::Or);
                    i += 2;
                } else {
                    tokens.push(Token::Pipe);
                    i += 1;
                }
            }
            '~' => {
                tokens.push(Token::Tilde);
                i += 1;
            }
            '$' => {
                tokens.push(Token::Dollar);
                i += 1;
            }
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
            }
            ';' => {
                tokens.push(Token::Semicolon);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '\\' => {
                // Line continuation
                if i + 1 < chars.len() && chars[i + 1] == '\n' {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                return Err(AwkError::Parse(format!("unexpected character: {:?}", c)).into());
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

// ============================================================
// AST
// ============================================================

#[derive(Debug, Clone)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Concat,
    Match,
    NotMatch,
}

#[derive(Debug, Clone)]
enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
enum Expr {
    Num(f64),
    Str(String),
    Regex(String),
    Var(String),
    FieldAccess(Box<Expr>),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    PreInc(Box<Expr>),
    PreDec(Box<Expr>),
    PostInc(Box<Expr>),
    PostDec(Box<Expr>),
    ArrayAccess(String, Box<Expr>),
    ArrayIn(Box<Expr>, String),
    FnCall(String, Vec<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
enum Stmt {
    Print(Vec<Expr>, Option<RedirTarget>),
    Printf(Vec<Expr>, Option<RedirTarget>),
    Assign(String, AssignOp, Expr),
    FieldAssign(Expr, Expr),
    ArrayAssign(String, Expr, AssignOp, Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Box<Stmt>>, Box<Stmt>),
    ForIn(String, String, Box<Stmt>),
    Do(Box<Stmt>, Expr),
    Block(Vec<Stmt>),
    Delete(String, Option<Expr>),
    Next,
    Exit(Option<Expr>),
    Return(Option<Expr>),
    Break,
    Continue,
    Expr(Expr),
}

#[derive(Debug, Clone)]
enum RedirTarget {
    File(Expr),
    Append(Expr),
}

#[derive(Debug, Clone)]
enum AssignOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone)]
enum Pattern {
    Begin,
    End,
    Expr(Expr),
    Range(Expr, Expr),
    Always,
}

#[derive(Debug, Clone)]
struct Rule {
    pattern: Pattern,
    action: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct Function {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct Program {
    rules: Vec<Rule>,
    functions: Vec<Function>,
}

// ============================================================
// Parser
// ============================================================

struct AwkParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> AwkParser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        AwkParser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek2(&self) -> &Token {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1]
        } else {
            &Token::Eof
        }
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline | Token::Semicolon) {
            self.advance();
        }
    }

    fn expect(&mut self, tok: &Token) -> Result<()> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(tok) {
            self.advance();
            Ok(())
        } else {
            Err(AwkError::Parse(format!(
                "expected {:?}, got {:?}",
                tok,
                self.peek()
            ))
            .into())
        }
    }

    fn consume_terminators(&mut self) {
        while matches!(self.peek(), Token::Newline | Token::Semicolon) {
            self.advance();
        }
    }

    fn parse_program(&mut self) -> Result<Program> {
        let mut rules = Vec::new();
        let mut functions = Vec::new();

        self.skip_newlines();

        while !matches!(self.peek(), Token::Eof) {
            self.skip_newlines();
            if matches!(self.peek(), Token::Eof) {
                break;
            }

            if matches!(self.peek(), Token::Function) {
                let f = self.parse_function()?;
                functions.push(f);
                continue;
            }

            let rule = self.parse_rule()?;
            rules.push(rule);
            self.skip_newlines();
        }

        Ok(Program { rules, functions })
    }

    fn parse_function(&mut self) -> Result<Function> {
        self.advance(); // consume 'function'
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            other => {
                return Err(
                    AwkError::Parse(format!("expected function name, got {:?}", other)).into(),
                )
            }
        };
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            if !params.is_empty() {
                self.expect(&Token::Comma)?;
            }
            match self.advance().clone() {
                Token::Ident(p) => params.push(p),
                other => {
                    return Err(AwkError::Parse(format!(
                        "expected param name, got {:?}",
                        other
                    ))
                    .into())
                }
            }
        }
        self.expect(&Token::RParen)?;
        self.skip_newlines();
        let body = self.parse_block()?;
        Ok(Function { name, params, body })
    }

    fn parse_rule(&mut self) -> Result<Rule> {
        let pattern = self.parse_pattern()?;
        self.skip_newlines();

        let action = if matches!(self.peek(), Token::LBrace) {
            self.parse_block()?
        } else if matches!(pattern, Pattern::Always) {
            return Err(AwkError::Parse("expected pattern or action".into()).into());
        } else {
            // Pattern without action => print $0
            vec![Stmt::Print(vec![Expr::FieldAccess(Box::new(Expr::Num(0.0)))], None)]
        };

        Ok(Rule { pattern, action })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.peek() {
            Token::Begin => {
                self.advance();
                Ok(Pattern::Begin)
            }
            Token::End => {
                self.advance();
                Ok(Pattern::End)
            }
            Token::LBrace => Ok(Pattern::Always),
            _ => {
                let expr = self.parse_expr()?;
                // Range pattern: expr, expr
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                    self.skip_newlines();
                    let expr2 = self.parse_expr()?;
                    Ok(Pattern::Range(expr, expr2))
                } else {
                    Ok(Pattern::Expr(expr))
                }
            }
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        self.expect(&Token::LBrace)?;
        self.skip_newlines();
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let s = self.parse_stmt()?;
            stmts.push(s);
            self.consume_terminators();
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek() {
            Token::LBrace => {
                let stmts = self.parse_block()?;
                Ok(Stmt::Block(stmts))
            }
            Token::If => {
                self.advance();
                self.expect(&Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                self.skip_newlines();
                let then_branch = self.parse_stmt()?;
                self.consume_terminators();
                let else_branch = if matches!(self.peek(), Token::Else) {
                    self.advance();
                    self.skip_newlines();
                    Some(Box::new(self.parse_stmt()?))
                } else {
                    None
                };
                Ok(Stmt::If(cond, Box::new(then_branch), else_branch))
            }
            Token::While => {
                self.advance();
                self.expect(&Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                self.skip_newlines();
                let body = self.parse_stmt()?;
                Ok(Stmt::While(cond, Box::new(body)))
            }
            Token::Do => {
                self.advance();
                self.skip_newlines();
                let body = self.parse_stmt()?;
                self.consume_terminators();
                self.expect(&Token::While)?;
                self.expect(&Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(Stmt::Do(Box::new(body), cond))
            }
            Token::For => {
                self.advance();
                self.expect(&Token::LParen)?;

                // Check for for(key in array) pattern
                // We need lookahead: if we see ident 'in' ident ')' it's a for-in
                let is_for_in = {
                    let save = self.pos;
                    let result = if let Token::Ident(_) = self.peek() {
                        self.advance();
                        if matches!(self.peek(), Token::In) {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    self.pos = save;
                    result
                };

                if is_for_in {
                    let var = match self.advance().clone() {
                        Token::Ident(v) => v,
                        _ => unreachable!(),
                    };
                    self.advance(); // consume 'in'
                    let arr = match self.advance().clone() {
                        Token::Ident(a) => a,
                        other => {
                            return Err(AwkError::Parse(format!(
                                "expected array name, got {:?}",
                                other
                            ))
                            .into())
                        }
                    };
                    self.expect(&Token::RParen)?;
                    self.skip_newlines();
                    let body = self.parse_stmt()?;
                    Ok(Stmt::ForIn(var, arr, Box::new(body)))
                } else {
                    // for(init; cond; incr)
                    let init = if matches!(self.peek(), Token::Semicolon) {
                        None
                    } else {
                        Some(Box::new(self.parse_stmt()?))
                    };
                    self.expect(&Token::Semicolon)?;
                    let cond = if matches!(self.peek(), Token::Semicolon) {
                        None
                    } else {
                        Some(self.parse_expr()?)
                    };
                    self.expect(&Token::Semicolon)?;
                    let incr = if matches!(self.peek(), Token::RParen) {
                        None
                    } else {
                        Some(Box::new(self.parse_stmt()?))
                    };
                    self.expect(&Token::RParen)?;
                    self.skip_newlines();
                    let body = self.parse_stmt()?;
                    Ok(Stmt::For(init, cond, incr, Box::new(body)))
                }
            }
            Token::Delete => {
                self.advance();
                match self.advance().clone() {
                    Token::Ident(arr) => {
                        if matches!(self.peek(), Token::LBracket) {
                            self.advance();
                            let key = self.parse_expr()?;
                            self.expect(&Token::RBracket)?;
                            Ok(Stmt::Delete(arr, Some(key)))
                        } else {
                            Ok(Stmt::Delete(arr, None))
                        }
                    }
                    other => Err(AwkError::Parse(format!(
                        "expected array name after delete, got {:?}",
                        other
                    ))
                    .into()),
                }
            }
            Token::Next => {
                self.advance();
                Ok(Stmt::Next)
            }
            Token::Exit => {
                self.advance();
                if matches!(
                    self.peek(),
                    Token::Newline | Token::Semicolon | Token::RBrace | Token::Eof
                ) {
                    Ok(Stmt::Exit(None))
                } else {
                    let e = self.parse_expr()?;
                    Ok(Stmt::Exit(Some(e)))
                }
            }
            Token::Return => {
                self.advance();
                if matches!(
                    self.peek(),
                    Token::Newline | Token::Semicolon | Token::RBrace | Token::Eof
                ) {
                    Ok(Stmt::Return(None))
                } else {
                    let e = self.parse_expr()?;
                    Ok(Stmt::Return(Some(e)))
                }
            }
            Token::Print => {
                self.advance();
                let exprs = self.parse_print_args()?;
                let redir = self.parse_optional_redir()?;
                Ok(Stmt::Print(exprs, redir))
            }
            Token::Printf => {
                self.advance();
                let exprs = self.parse_print_args()?;
                let redir = self.parse_optional_redir()?;
                Ok(Stmt::Printf(exprs, redir))
            }
            Token::Ident(_) => {
                // Could be assignment or expression statement
                // Peek further to determine
                self.parse_assign_or_expr_stmt()
            }
            _ => {
                let e = self.parse_expr()?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn parse_print_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        if matches!(
            self.peek(),
            Token::Newline
                | Token::Semicolon
                | Token::RBrace
                | Token::Eof
                | Token::Gt
                | Token::Append
                | Token::Pipe
        ) {
            return Ok(args);
        }
        // print with parens: treat content as arg list
        if matches!(self.peek(), Token::LParen) {
            // Could be print (expr) or print (expr, expr, ...)
            // To avoid ambiguity, parse normally
            args.push(self.parse_concat_expr()?);
            while matches!(self.peek(), Token::Comma) {
                self.advance();
                args.push(self.parse_concat_expr()?);
            }
        } else {
            args.push(self.parse_concat_expr()?);
            while matches!(self.peek(), Token::Comma) {
                self.advance();
                args.push(self.parse_concat_expr()?);
            }
        }
        Ok(args)
    }

    fn parse_optional_redir(&mut self) -> Result<Option<RedirTarget>> {
        match self.peek() {
            Token::Gt => {
                self.advance();
                let e = self.parse_concat_expr()?;
                Ok(Some(RedirTarget::File(e)))
            }
            Token::Append => {
                self.advance();
                let e = self.parse_concat_expr()?;
                Ok(Some(RedirTarget::Append(e)))
            }
            _ => Ok(None),
        }
    }

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt> {
        // Look ahead to see if this is an assignment
        let save = self.pos;
        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.pos = save;
                let e = self.parse_expr()?;
                return Ok(Stmt::Expr(e));
            }
        };

        // Check for array assignment: ident[expr] op= expr
        if matches!(self.peek(), Token::LBracket) {
            self.advance();
            let key = self.parse_expr()?;
            self.expect(&Token::RBracket)?;
            let op = match self.peek() {
                Token::Assign => {
                    self.advance();
                    AssignOp::Set
                }
                Token::PlusEq => {
                    self.advance();
                    AssignOp::Add
                }
                Token::MinusEq => {
                    self.advance();
                    AssignOp::Sub
                }
                Token::StarEq => {
                    self.advance();
                    AssignOp::Mul
                }
                Token::SlashEq => {
                    self.advance();
                    AssignOp::Div
                }
                Token::PercentEq => {
                    self.advance();
                    AssignOp::Mod
                }
                _ => {
                    // Not an assignment — restore and parse as expr
                    self.pos = save;
                    let e = self.parse_expr()?;
                    return Ok(Stmt::Expr(e));
                }
            };
            let val = self.parse_expr()?;
            return Ok(Stmt::ArrayAssign(name, key, op, val));
        }

        // Check for simple assignment: ident op= expr
        let op = match self.peek() {
            Token::Assign => {
                self.advance();
                AssignOp::Set
            }
            Token::PlusEq => {
                self.advance();
                AssignOp::Add
            }
            Token::MinusEq => {
                self.advance();
                AssignOp::Sub
            }
            Token::StarEq => {
                self.advance();
                AssignOp::Mul
            }
            Token::SlashEq => {
                self.advance();
                AssignOp::Div
            }
            Token::PercentEq => {
                self.advance();
                AssignOp::Mod
            }
            _ => {
                // Not an assignment — restore and parse as expr
                self.pos = save;
                let e = self.parse_expr()?;
                return Ok(Stmt::Expr(e));
            }
        };
        let val = self.parse_expr()?;
        Ok(Stmt::Assign(name, op, val))
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expr> {
        let cond = self.parse_or()?;
        if matches!(self.peek(), Token::Gt) {
            // Could be ternary ? but we don't have ? token...
            // AWK doesn't actually have ?:, skip
            Ok(cond)
        } else {
            Ok(cond)
        }
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_match()?;
        while matches!(self.peek(), Token::And) {
            self.advance();
            let right = self.parse_match()?;
            left = Expr::BinOp(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_match(&mut self) -> Result<Expr> {
        let left = self.parse_in()?;
        match self.peek() {
            Token::Tilde => {
                self.advance();
                let right = self.parse_in()?;
                Ok(Expr::BinOp(Box::new(left), BinOp::Match, Box::new(right)))
            }
            Token::NotTilde => {
                self.advance();
                let right = self.parse_in()?;
                Ok(Expr::BinOp(
                    Box::new(left),
                    BinOp::NotMatch,
                    Box::new(right),
                ))
            }
            _ => Ok(left),
        }
    }

    fn parse_in(&mut self) -> Result<Expr> {
        let left = self.parse_comparison()?;
        if matches!(self.peek(), Token::In) {
            self.advance();
            if let Token::Ident(arr) = self.peek().clone() {
                self.advance();
                return Ok(Expr::ArrayIn(Box::new(left), arr));
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_concat_expr()?;
        loop {
            let op = match self.peek() {
                Token::Eq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat_expr()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_concat_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;
        // String concatenation: two adjacent expressions
        loop {
            // If next token can start an expression but is NOT an operator
            // then it's implicit concatenation
            if self.is_concat_start() {
                let right = self.parse_additive()?;
                left = Expr::BinOp(Box::new(left), BinOp::Concat, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn is_concat_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Number(_)
                | Token::Str(_)
                | Token::Ident(_)
                | Token::Dollar
                | Token::LParen
                | Token::Bang
                | Token::Minus
                | Token::Plus
        )
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr> {
        let left = self.parse_unary()?;
        if matches!(self.peek(), Token::Caret) {
            self.advance();
            let right = self.parse_power()?; // right-associative
            Ok(Expr::BinOp(Box::new(left), BinOp::Pow, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let e = self.parse_unary()?;
                Ok(Expr::UnOp(UnOp::Neg, Box::new(e)))
            }
            Token::Plus => {
                self.advance();
                self.parse_unary()
            }
            Token::Bang => {
                self.advance();
                let e = self.parse_unary()?;
                Ok(Expr::UnOp(UnOp::Not, Box::new(e)))
            }
            Token::PlusPlus => {
                self.advance();
                let e = self.parse_postfix()?;
                Ok(Expr::PreInc(Box::new(e)))
            }
            Token::MinusMinus => {
                self.advance();
                let e = self.parse_postfix()?;
                Ok(Expr::PreDec(Box::new(e)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let e = self.parse_primary()?;
        match self.peek() {
            Token::PlusPlus => {
                self.advance();
                Ok(Expr::PostInc(Box::new(e)))
            }
            Token::MinusMinus => {
                self.advance();
                Ok(Expr::PostDec(Box::new(e)))
            }
            _ => Ok(e),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Num(n))
            }
            Token::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            Token::Regex(s) => {
                self.advance();
                Ok(Expr::Regex(s))
            }
            Token::Dollar => {
                self.advance();
                let inner = self.parse_primary()?;
                Ok(Expr::FieldAccess(Box::new(inner)))
            }
            Token::Ident(name) => {
                self.advance();
                // Function call?
                if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.peek(), Token::RParen | Token::Eof) {
                        if !args.is_empty() {
                            self.expect(&Token::Comma)?;
                        }
                        args.push(self.parse_expr()?);
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::FnCall(name, args))
                } else if matches!(self.peek(), Token::LBracket) {
                    // Array access
                    self.advance();
                    let key = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    Ok(Expr::ArrayAccess(name, Box::new(key)))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            other => {
                Err(AwkError::Parse(format!("unexpected token in expression: {:?}", other)).into())
            }
        }
    }
}

fn parse(tokens: &[Token]) -> Result<Program> {
    let mut parser = AwkParser::new(tokens);
    parser.parse_program()
}

// ============================================================
// Runtime Value
// ============================================================

#[derive(Debug, Clone)]
enum Value {
    Num(f64),
    Str(String),
    Uninitialized,
}

impl Value {
    fn to_num(&self) -> f64 {
        match self {
            Value::Num(n) => *n,
            Value::Str(s) => {
                // Parse leading number from string
                let s = s.trim();
                if s.is_empty() {
                    return 0.0;
                }
                // Find longest leading numeric prefix
                let end = s
                    .find(|c: char| !c.is_ascii_digit() && c != '.' && c != 'e' && c != 'E' && c != '+' && c != '-')
                    .unwrap_or(s.len());
                s[..end].parse::<f64>().unwrap_or(0.0)
            }
            Value::Uninitialized => 0.0,
        }
    }

    fn to_str(&self) -> String {
        match self {
            Value::Num(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Str(s) => s.clone(),
            Value::Uninitialized => String::new(),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Num(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Uninitialized => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

// ============================================================
// Control Flow
// ============================================================

#[derive(Debug)]
enum ControlFlow {
    Normal,
    Next,
    Exit(i32),
    Break,
    Continue,
    Return(Value),
}

// ============================================================
// Interpreter Environment
// ============================================================

struct Env {
    globals: HashMap<String, Value>,
    arrays: HashMap<String, HashMap<String, Value>>,
    locals_stack: Vec<HashMap<String, Value>>,
    local_arrays_stack: Vec<HashMap<String, HashMap<String, Value>>>,
    fields: Vec<String>,
    nr: usize,
    fnr: usize,
    nf: usize,
    fs: String,
    ofs: String,
    rs: String,
    ors: String,
    filename: String,
    // Output files (for redirect support - security: T-04-07-03 omit file writes initially)
    // Per threat model T-04-07-03: redirect not supported in this implementation
    range_active: HashMap<usize, bool>, // for range patterns
}

impl Env {
    fn new(fs: &str, assigns: &[String]) -> Result<Self> {
        let mut env = Env {
            globals: HashMap::new(),
            arrays: HashMap::new(),
            locals_stack: Vec::new(),
            local_arrays_stack: Vec::new(),
            fields: Vec::new(),
            nr: 0,
            fnr: 0,
            nf: 0,
            fs: fs.to_string(),
            ofs: " ".to_string(),
            rs: "\n".to_string(),
            ors: "\n".to_string(),
            filename: String::new(),
            range_active: HashMap::new(),
        };

        // Process -v assignments (T-04-07-06 mitigation: validate VAR names)
        for a in assigns {
            let eq_pos = a
                .find('=')
                .ok_or_else(|| AwkError::InvalidAssignment(a.clone()))?;
            let var = &a[..eq_pos];
            let val = &a[eq_pos + 1..];

            // Validate variable name (T-04-07-06)
            if !var.chars().all(|c| c.is_alphanumeric() || c == '_')
                || var.starts_with(|c: char| c.is_ascii_digit())
            {
                return Err(AwkError::InvalidAssignment(a.clone()).into());
            }

            env.globals
                .insert(var.to_string(), Value::Str(val.to_string()));
        }

        Ok(env)
    }

    fn get_var(&self, name: &str) -> Value {
        // Check locals first
        for frame in self.locals_stack.iter().rev() {
            if let Some(v) = frame.get(name) {
                return v.clone();
            }
        }
        // Built-in variables
        match name {
            "NR" => Value::Num(self.nr as f64),
            "NF" => Value::Num(self.nf as f64),
            "FNR" => Value::Num(self.fnr as f64),
            "FS" => Value::Str(self.fs.clone()),
            "OFS" => Value::Str(self.ofs.clone()),
            "RS" => Value::Str(self.rs.clone()),
            "ORS" => Value::Str(self.ors.clone()),
            "FILENAME" => Value::Str(self.filename.clone()),
            _ => self
                .globals
                .get(name)
                .cloned()
                .unwrap_or(Value::Uninitialized),
        }
    }

    fn set_var(&mut self, name: &str, val: Value) {
        // Check locals first
        if let Some(frame) = self.locals_stack.last_mut() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), val);
                return;
            }
        }
        // Built-in variable assignments
        match name {
            "FS" => self.fs = val.to_str(),
            "OFS" => self.ofs = val.to_str(),
            "RS" => self.rs = val.to_str(),
            "ORS" => self.ors = val.to_str(),
            "NF" => {
                let n = val.to_num() as usize;
                self.nf = n;
                self.fields.resize(n + 1, String::new());
            }
            _ => {
                self.globals.insert(name.to_string(), val);
            }
        }
    }

    fn get_field(&self, idx: usize) -> Value {
        if idx == 0 {
            Value::Str(self.fields.get(0).cloned().unwrap_or_default())
        } else if idx <= self.nf {
            Value::Str(self.fields.get(idx).cloned().unwrap_or_default())
        } else {
            Value::Uninitialized
        }
    }

    fn set_field(&mut self, idx: usize, val: String) {
        if idx == 0 {
            // Setting $0 re-splits fields
            let record = val;
            self.fields = split_fields(&record, &self.fs);
            self.nf = self.fields.len() - 1;
        } else {
            // Extend fields if needed
            while self.fields.len() <= idx {
                self.fields.push(String::new());
            }
            self.fields[idx] = val;
            if idx > self.nf {
                self.nf = idx;
            }
            // Rebuild $0
            let reconstructed = self.fields[1..].join(&self.ofs);
            self.fields[0] = reconstructed;
        }
    }

    fn get_array_val(&self, arr: &str, key: &str) -> Value {
        // Check local arrays first
        for frame in self.local_arrays_stack.iter().rev() {
            if let Some(a) = frame.get(arr) {
                return a.get(key).cloned().unwrap_or(Value::Uninitialized);
            }
        }
        self.arrays
            .get(arr)
            .and_then(|a| a.get(key))
            .cloned()
            .unwrap_or(Value::Uninitialized)
    }

    fn set_array_val(&mut self, arr: &str, key: &str, val: Value) {
        // Check local arrays first
        if let Some(frame) = self.local_arrays_stack.last_mut() {
            if frame.contains_key(arr) {
                frame
                    .entry(arr.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(key.to_string(), val);
                return;
            }
        }
        self.arrays
            .entry(arr.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), val);
    }

    fn set_record(&mut self, record: &str) {
        self.fields = split_fields(record, &self.fs);
        self.nf = self.fields.len() - 1;
    }
}

// ============================================================
// Field Splitting
// ============================================================

fn split_fields(record: &str, fs: &str) -> Vec<String> {
    let mut result = vec![record.to_string()]; // fields[0] = $0

    if fs == " " {
        // Default: split on whitespace runs, ignore leading/trailing
        let parts: Vec<String> = record.split_whitespace().map(|s| s.to_string()).collect();
        result.extend(parts);
    } else if fs.len() == 1 {
        // Single character separator
        let ch = fs.chars().next().unwrap();
        let parts: Vec<String> = record.split(ch).map(|s| s.to_string()).collect();
        result.extend(parts);
    } else {
        // Multi-char regex separator
        match Regex::new(fs) {
            Ok(re) => {
                let parts: Vec<String> = re.split(record).map(|s| s.to_string()).collect();
                result.extend(parts);
            }
            Err(_) => {
                // Fall back to literal split
                let parts: Vec<String> = record.split(fs).map(|s| s.to_string()).collect();
                result.extend(parts);
            }
        }
    }

    result
}

// ============================================================
// printf / sprintf formatting
// ============================================================

fn format_printf(fmt: &str, args: &[Value]) -> String {
    let mut result = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    let mut arg_idx = 0;

    while i < chars.len() {
        if chars[i] != '%' {
            result.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= chars.len() {
            break;
        }
        if chars[i] == '%' {
            result.push('%');
            i += 1;
            continue;
        }

        // Parse flags
        let mut flags = String::new();
        while i < chars.len() && "-+0 #".contains(chars[i]) {
            flags.push(chars[i]);
            i += 1;
        }

        // Width
        let mut width_str = String::new();
        while i < chars.len() && chars[i].is_ascii_digit() {
            width_str.push(chars[i]);
            i += 1;
        }
        let width: usize = width_str.parse().unwrap_or(0);

        // Precision
        let mut precision: Option<usize> = None;
        if i < chars.len() && chars[i] == '.' {
            i += 1;
            let mut prec_str = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                prec_str.push(chars[i]);
                i += 1;
            }
            precision = Some(prec_str.parse().unwrap_or(0));
        }

        if i >= chars.len() {
            break;
        }

        let spec = chars[i];
        i += 1;

        let arg = if arg_idx < args.len() {
            args[arg_idx].clone()
        } else {
            Value::Uninitialized
        };
        arg_idx += 1;

        let left_align = flags.contains('-');
        let zero_pad = flags.contains('0') && !left_align;
        let plus_sign = flags.contains('+');

        let formatted = match spec {
            'd' | 'i' => {
                let n = arg.to_num() as i64;
                let s = if plus_sign && n >= 0 {
                    format!("+{}", n)
                } else {
                    format!("{}", n)
                };
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'u' => {
                let n = arg.to_num() as u64;
                let s = format!("{}", n);
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'f' | 'F' => {
                let n = arg.to_num();
                let prec = precision.unwrap_or(6);
                let s = if plus_sign && n >= 0.0 {
                    format!("+{:.prec$}", n, prec = prec)
                } else {
                    format!("{:.prec$}", n, prec = prec)
                };
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'e' => {
                let n = arg.to_num();
                let prec = precision.unwrap_or(6);
                let s = format_scientific(n, prec, false);
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'E' => {
                let n = arg.to_num();
                let prec = precision.unwrap_or(6);
                let s = format_scientific(n, prec, true);
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'g' | 'G' => {
                let n = arg.to_num();
                let prec = precision.unwrap_or(6);
                let prec = if prec == 0 { 1 } else { prec };
                let s = format_g(n, prec, spec == 'G');
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'x' => {
                let n = arg.to_num() as i64 as u64;
                let s = format!("{:x}", n);
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'X' => {
                let n = arg.to_num() as i64 as u64;
                let s = format!("{:X}", n);
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            'o' => {
                let n = arg.to_num() as i64 as u64;
                let s = format!("{:o}", n);
                pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' })
            }
            's' => {
                let s = arg.to_str();
                let s = if let Some(prec) = precision {
                    if prec < s.len() {
                        s[..prec].to_string()
                    } else {
                        s
                    }
                } else {
                    s
                };
                pad_string(&s, width, left_align, ' ')
            }
            'c' => {
                let s = arg.to_str();
                let ch = if s.is_empty() {
                    '\0'
                } else {
                    s.chars().next().unwrap()
                };
                let s = ch.to_string();
                pad_string(&s, width, left_align, ' ')
            }
            _ => {
                // Unknown format spec, pass through
                format!("%{}", spec)
            }
        };

        result.push_str(&formatted);
    }

    result
}

fn pad_string(s: &str, width: usize, left_align: bool, pad_char: char) -> String {
    if s.len() >= width {
        return s.to_string();
    }
    let padding: String = std::iter::repeat(pad_char).take(width - s.len()).collect();
    if left_align {
        format!("{}{}", s, padding)
    } else if pad_char == '0' && (s.starts_with('-') || s.starts_with('+')) {
        // For zero-padding with sign, put zeros after sign
        format!("{}{}{}", &s[..1], padding, &s[1..])
    } else {
        format!("{}{}", padding, s)
    }
}

fn format_scientific(n: f64, prec: usize, upper: bool) -> String {
    if n == 0.0 {
        let e_char = if upper { 'E' } else { 'e' };
        return format!("{:.prec$}{}{:+03}", 0.0, e_char, 0, prec = prec);
    }
    let exp = n.abs().log10().floor() as i32;
    let mantissa = n / 10f64.powi(exp);
    let e_char = if upper { 'E' } else { 'e' };
    format!(
        "{:.prec$}{}{:+03}",
        mantissa,
        e_char,
        exp,
        prec = prec
    )
}

fn format_g(n: f64, prec: usize, upper: bool) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    let exp = n.abs().log10().floor() as i32;
    if exp >= -(4i32) && exp < prec as i32 {
        // Use fixed notation
        let decimal_places = (prec as i32 - 1 - exp).max(0) as usize;
        let s = format!("{:.prec$}", n, prec = decimal_places);
        // Remove trailing zeros
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    } else {
        let s = format_scientific(n, prec - 1, upper);
        s
    }
}

// ============================================================
// Evaluator
// ============================================================

struct Interpreter<'a> {
    program: &'a Program,
    functions: HashMap<String, &'a Function>,
}

impl<'a> Interpreter<'a> {
    fn new(program: &'a Program) -> Self {
        let mut functions = HashMap::new();
        for f in &program.functions {
            functions.insert(f.name.clone(), f);
        }
        Interpreter { program, functions }
    }

    fn eval_expr(&self, expr: &Expr, env: &mut Env) -> Result<Value> {
        match expr {
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Regex(s) => {
                // Bare regex in expression context — matches against $0
                let re = Regex::new(s)
                    .map_err(|e| AwkError::Runtime(format!("invalid regex {:?}: {}", s, e)))?;
                let field0 = env.get_field(0).to_str();
                Ok(Value::Num(if re.is_match(&field0) { 1.0 } else { 0.0 }))
            }
            Expr::Var(name) => Ok(env.get_var(name)),
            Expr::FieldAccess(idx_expr) => {
                let idx = self.eval_expr(idx_expr, env)?.to_num() as usize;
                Ok(env.get_field(idx))
            }
            Expr::ArrayAccess(arr, key_expr) => {
                let key = self.eval_expr(key_expr, env)?.to_str();
                Ok(env.get_array_val(arr, &key))
            }
            Expr::ArrayIn(key_expr, arr) => {
                let key = self.eval_expr(key_expr, env)?.to_str();
                let exists = env.arrays.get(arr).map_or(false, |a| a.contains_key(&key));
                Ok(Value::Num(if exists { 1.0 } else { 0.0 }))
            }
            Expr::BinOp(left, op, right) => self.eval_binop(left, op, right, env),
            Expr::UnOp(op, expr) => {
                let val = self.eval_expr(expr, env)?;
                match op {
                    UnOp::Neg => Ok(Value::Num(-val.to_num())),
                    UnOp::Not => Ok(Value::Num(if val.is_truthy() { 0.0 } else { 1.0 })),
                }
            }
            Expr::PreInc(expr) => {
                let new_val = self.eval_expr(expr, env)?.to_num() + 1.0;
                self.assign_lvalue(expr, Value::Num(new_val), env)?;
                Ok(Value::Num(new_val))
            }
            Expr::PreDec(expr) => {
                let new_val = self.eval_expr(expr, env)?.to_num() - 1.0;
                self.assign_lvalue(expr, Value::Num(new_val), env)?;
                Ok(Value::Num(new_val))
            }
            Expr::PostInc(expr) => {
                let old_val = self.eval_expr(expr, env)?.to_num();
                self.assign_lvalue(expr, Value::Num(old_val + 1.0), env)?;
                Ok(Value::Num(old_val))
            }
            Expr::PostDec(expr) => {
                let old_val = self.eval_expr(expr, env)?.to_num();
                self.assign_lvalue(expr, Value::Num(old_val - 1.0), env)?;
                Ok(Value::Num(old_val))
            }
            Expr::FnCall(name, args) => self.eval_fn_call(name, args, env),
            Expr::Ternary(cond, then_expr, else_expr) => {
                let cv = self.eval_expr(cond, env)?;
                if cv.is_truthy() {
                    self.eval_expr(then_expr, env)
                } else {
                    self.eval_expr(else_expr, env)
                }
            }
        }
    }

    fn assign_lvalue(&self, expr: &Expr, val: Value, env: &mut Env) -> Result<()> {
        match expr {
            Expr::Var(name) => {
                env.set_var(name, val);
            }
            Expr::FieldAccess(idx_expr) => {
                let idx = self.eval_expr(idx_expr, env)?.to_num() as usize;
                env.set_field(idx, val.to_str());
            }
            Expr::ArrayAccess(arr, key_expr) => {
                let key = self.eval_expr(key_expr, env)?.to_str();
                env.set_array_val(arr, &key, val);
            }
            _ => {
                return Err(AwkError::Runtime("invalid lvalue".into()).into());
            }
        }
        Ok(())
    }

    fn eval_binop(&self, left: &Expr, op: &BinOp, right: &Expr, env: &mut Env) -> Result<Value> {
        // Short-circuit for And/Or
        match op {
            BinOp::And => {
                let lv = self.eval_expr(left, env)?;
                if !lv.is_truthy() {
                    return Ok(Value::Num(0.0));
                }
                let rv = self.eval_expr(right, env)?;
                return Ok(Value::Num(if rv.is_truthy() { 1.0 } else { 0.0 }));
            }
            BinOp::Or => {
                let lv = self.eval_expr(left, env)?;
                if lv.is_truthy() {
                    return Ok(Value::Num(1.0));
                }
                let rv = self.eval_expr(right, env)?;
                return Ok(Value::Num(if rv.is_truthy() { 1.0 } else { 0.0 }));
            }
            _ => {}
        }

        let lv = self.eval_expr(left, env)?;
        let rv = self.eval_expr(right, env)?;

        match op {
            BinOp::Add => Ok(Value::Num(lv.to_num() + rv.to_num())),
            BinOp::Sub => Ok(Value::Num(lv.to_num() - rv.to_num())),
            BinOp::Mul => Ok(Value::Num(lv.to_num() * rv.to_num())),
            BinOp::Div => {
                let divisor = rv.to_num();
                if divisor == 0.0 {
                    return Err(AwkError::Runtime("division by zero".into()).into());
                }
                Ok(Value::Num(lv.to_num() / divisor))
            }
            BinOp::Mod => {
                let divisor = rv.to_num();
                if divisor == 0.0 {
                    return Err(AwkError::Runtime("modulo by zero".into()).into());
                }
                Ok(Value::Num(lv.to_num() % divisor))
            }
            BinOp::Pow => Ok(Value::Num(lv.to_num().powf(rv.to_num()))),
            BinOp::Concat => Ok(Value::Str(format!("{}{}", lv.to_str(), rv.to_str()))),
            BinOp::Eq => Ok(Value::Num(compare_values(&lv, &rv, |a, b| a == b, |a, b| a == b))),
            BinOp::Ne => Ok(Value::Num(compare_values(&lv, &rv, |a, b| a != b, |a, b| a != b))),
            BinOp::Lt => Ok(Value::Num(compare_values(&lv, &rv, |a, b| a < b, |a, b| a < b))),
            BinOp::Le => Ok(Value::Num(compare_values(&lv, &rv, |a, b| a <= b, |a, b| a <= b))),
            BinOp::Gt => Ok(Value::Num(compare_values(&lv, &rv, |a, b| a > b, |a, b| a > b))),
            BinOp::Ge => Ok(Value::Num(compare_values(&lv, &rv, |a, b| a >= b, |a, b| a >= b))),
            BinOp::Match => {
                let pat = rv.to_str();
                let re = Regex::new(&pat)
                    .map_err(|e| AwkError::Runtime(format!("invalid regex {:?}: {}", pat, e)))?;
                Ok(Value::Num(if re.is_match(&lv.to_str()) { 1.0 } else { 0.0 }))
            }
            BinOp::NotMatch => {
                let pat = rv.to_str();
                let re = Regex::new(&pat)
                    .map_err(|e| AwkError::Runtime(format!("invalid regex {:?}: {}", pat, e)))?;
                Ok(Value::Num(if re.is_match(&lv.to_str()) { 0.0 } else { 1.0 }))
            }
            BinOp::And | BinOp::Or => unreachable!(), // handled above
        }
    }

    fn eval_fn_call(&self, name: &str, args: &[Expr], env: &mut Env) -> Result<Value> {
        match name {
            "length" => {
                if args.is_empty() {
                    // length with no args = length($0)
                    return Ok(Value::Num(env.get_field(0).to_str().len() as f64));
                }
                // Check if arg is an array name
                if let Some(Expr::Var(arr_name)) = args.first() {
                    if env.arrays.contains_key(arr_name.as_str()) {
                        return Ok(Value::Num(env.arrays[arr_name.as_str()].len() as f64));
                    }
                }
                let s = self.eval_expr(&args[0], env)?.to_str();
                Ok(Value::Num(s.chars().count() as f64))
            }
            "substr" => {
                if args.len() < 2 {
                    return Err(
                        AwkError::Runtime("substr requires at least 2 args".into()).into(),
                    );
                }
                let s = self.eval_expr(&args[0], env)?.to_str();
                let start = (self.eval_expr(&args[1], env)?.to_num() as isize - 1).max(0) as usize;
                let chars: Vec<char> = s.chars().collect();
                if start >= chars.len() {
                    return Ok(Value::Str(String::new()));
                }
                if args.len() >= 3 {
                    let len = self.eval_expr(&args[2], env)?.to_num() as usize;
                    let end = (start + len).min(chars.len());
                    Ok(Value::Str(chars[start..end].iter().collect()))
                } else {
                    Ok(Value::Str(chars[start..].iter().collect()))
                }
            }
            "index" => {
                if args.len() < 2 {
                    return Ok(Value::Num(0.0));
                }
                let s = self.eval_expr(&args[0], env)?.to_str();
                let t = self.eval_expr(&args[1], env)?.to_str();
                if let Some(pos) = s.find(&t as &str) {
                    // Count chars up to pos
                    let char_pos = s[..pos].chars().count() + 1;
                    Ok(Value::Num(char_pos as f64))
                } else {
                    Ok(Value::Num(0.0))
                }
            }
            "split" => {
                if args.len() < 2 {
                    return Err(AwkError::Runtime("split requires at least 2 args".into()).into());
                }
                let s = self.eval_expr(&args[0], env)?.to_str();
                let arr_name = match &args[1] {
                    Expr::Var(n) => n.clone(),
                    _ => {
                        return Err(
                            AwkError::Runtime("split: second arg must be array name".into()).into(),
                        )
                    }
                };
                let sep = if args.len() >= 3 {
                    self.eval_expr(&args[2], env)?.to_str()
                } else {
                    env.fs.clone()
                };
                // Clear the array
                env.arrays.insert(arr_name.clone(), HashMap::new());
                let fields = split_fields(&s, &sep);
                let count = fields.len() - 1; // fields[0] is the full string
                for (i, f) in fields[1..].iter().enumerate() {
                    env.set_array_val(&arr_name, &(i + 1).to_string(), Value::Str(f.clone()));
                }
                Ok(Value::Num(count as f64))
            }
            "sub" => {
                if args.len() < 2 {
                    return Err(AwkError::Runtime("sub requires at least 2 args".into()).into());
                }
                let pat = self.eval_expr(&args[0], env)?.to_str();
                let repl = self.eval_expr(&args[1], env)?.to_str();
                let target_val = if args.len() >= 3 {
                    self.eval_expr(&args[2], env)?.to_str()
                } else {
                    env.get_field(0).to_str()
                };
                let re = Regex::new(&pat)
                    .map_err(|e| AwkError::Runtime(format!("invalid regex: {}", e)))?;
                let count = if re.is_match(&target_val) { 1 } else { 0 };
                let new_val = re.replacen(&target_val, 1, repl.as_str()).to_string();
                if args.len() >= 3 {
                    self.assign_lvalue(&args[2], Value::Str(new_val), env)?;
                } else {
                    env.set_field(0, new_val);
                }
                Ok(Value::Num(count as f64))
            }
            "gsub" => {
                if args.len() < 2 {
                    return Err(AwkError::Runtime("gsub requires at least 2 args".into()).into());
                }
                let pat = self.eval_expr(&args[0], env)?.to_str();
                let repl = self.eval_expr(&args[1], env)?.to_str();
                let target_val = if args.len() >= 3 {
                    self.eval_expr(&args[2], env)?.to_str()
                } else {
                    env.get_field(0).to_str()
                };
                let re = Regex::new(&pat)
                    .map_err(|e| AwkError::Runtime(format!("invalid regex: {}", e)))?;
                let count = re.find_iter(&target_val).count();
                let new_val = re.replace_all(&target_val, repl.as_str()).to_string();
                if args.len() >= 3 {
                    self.assign_lvalue(&args[2], Value::Str(new_val), env)?;
                } else {
                    env.set_field(0, new_val);
                }
                Ok(Value::Num(count as f64))
            }
            "match" => {
                if args.len() < 2 {
                    return Ok(Value::Num(0.0));
                }
                let s = self.eval_expr(&args[0], env)?.to_str();
                let pat = self.eval_expr(&args[1], env)?.to_str();
                let re = Regex::new(&pat)
                    .map_err(|e| AwkError::Runtime(format!("invalid regex: {}", e)))?;
                if let Some(m) = re.find(&s) {
                    let start = s[..m.start()].chars().count() + 1;
                    let len = m.as_str().chars().count();
                    env.set_var("RSTART", Value::Num(start as f64));
                    env.set_var("RLENGTH", Value::Num(len as f64));
                    Ok(Value::Num(start as f64))
                } else {
                    env.set_var("RSTART", Value::Num(0.0));
                    env.set_var("RLENGTH", Value::Num(-1.0));
                    Ok(Value::Num(0.0))
                }
            }
            "toupper" => {
                if args.is_empty() {
                    return Ok(Value::Str(String::new()));
                }
                let s = self.eval_expr(&args[0], env)?.to_str();
                Ok(Value::Str(s.to_uppercase()))
            }
            "tolower" => {
                if args.is_empty() {
                    return Ok(Value::Str(String::new()));
                }
                let s = self.eval_expr(&args[0], env)?.to_str();
                Ok(Value::Str(s.to_lowercase()))
            }
            "sprintf" => {
                if args.is_empty() {
                    return Ok(Value::Str(String::new()));
                }
                let fmt = self.eval_expr(&args[0], env)?.to_str();
                let mut vals = Vec::new();
                for a in &args[1..] {
                    vals.push(self.eval_expr(a, env)?);
                }
                Ok(Value::Str(format_printf(&fmt, &vals)))
            }
            "int" => {
                if args.is_empty() {
                    return Ok(Value::Num(0.0));
                }
                let n = self.eval_expr(&args[0], env)?.to_num();
                Ok(Value::Num(n.trunc()))
            }
            "sqrt" => {
                if args.is_empty() {
                    return Ok(Value::Num(0.0));
                }
                let n = self.eval_expr(&args[0], env)?.to_num();
                Ok(Value::Num(n.sqrt()))
            }
            "sin" => {
                if args.is_empty() {
                    return Ok(Value::Num(0.0));
                }
                let n = self.eval_expr(&args[0], env)?.to_num();
                Ok(Value::Num(n.sin()))
            }
            "cos" => {
                if args.is_empty() {
                    return Ok(Value::Num(0.0));
                }
                let n = self.eval_expr(&args[0], env)?.to_num();
                Ok(Value::Num(n.cos()))
            }
            "atan2" => {
                if args.len() < 2 {
                    return Ok(Value::Num(0.0));
                }
                let y = self.eval_expr(&args[0], env)?.to_num();
                let x = self.eval_expr(&args[1], env)?.to_num();
                Ok(Value::Num(y.atan2(x)))
            }
            "exp" => {
                if args.is_empty() {
                    return Ok(Value::Num(1.0));
                }
                let n = self.eval_expr(&args[0], env)?.to_num();
                Ok(Value::Num(n.exp()))
            }
            "log" => {
                if args.is_empty() {
                    return Ok(Value::Num(0.0));
                }
                let n = self.eval_expr(&args[0], env)?.to_num();
                Ok(Value::Num(n.ln()))
            }
            "system" => {
                // T-04-07-04: system() is intentionally not supported
                Err(AwkError::Runtime(
                    "system() not supported (security restriction)".into(),
                )
                .into())
            }
            "print" | "printf" => {
                // These are statements, not functions — shouldn't appear here
                Err(AwkError::Runtime(format!(
                    "{} used as function — use as statement",
                    name
                ))
                .into())
            }
            _ => {
                // User-defined function call
                if let Some(func) = self.functions.get(name) {
                    let func = *func;
                    let mut locals: HashMap<String, Value> = HashMap::new();
                    let mut local_arrays: HashMap<String, HashMap<String, Value>> = HashMap::new();

                    // Bind parameters
                    for (i, param) in func.params.iter().enumerate() {
                        let val = if i < args.len() {
                            self.eval_expr(&args[i], env)?
                        } else {
                            Value::Uninitialized
                        };
                        locals.insert(param.clone(), val);
                    }

                    env.locals_stack.push(locals);
                    env.local_arrays_stack.push(local_arrays);

                    let mut result = Value::Uninitialized;
                    for stmt in &func.body {
                        match self.exec_stmt(stmt, env)? {
                            ControlFlow::Return(v) => {
                                result = v;
                                break;
                            }
                            ControlFlow::Exit(code) => {
                                env.locals_stack.pop();
                                env.local_arrays_stack.pop();
                                return Err(anyhow!("__exit__{}", code));
                            }
                            _ => {}
                        }
                    }

                    env.locals_stack.pop();
                    env.local_arrays_stack.pop();
                    Ok(result)
                } else {
                    Err(AwkError::Runtime(format!("undefined function: {}", name)).into())
                }
            }
        }
    }

    fn exec_stmt(&self, stmt: &Stmt, env: &mut Env) -> Result<ControlFlow> {
        match stmt {
            Stmt::Block(stmts) => {
                for s in stmts {
                    let cf = self.exec_stmt(s, env)?;
                    if !matches!(cf, ControlFlow::Normal) {
                        return Ok(cf);
                    }
                }
                Ok(ControlFlow::Normal)
            }
            Stmt::Print(exprs, redir) => {
                // T-04-07-03: Redirect to files disabled except for stderr/stdout
                if let Some(RedirTarget::File(_)) | Some(RedirTarget::Append(_)) = redir {
                    return Err(AwkError::Runtime(
                        "print redirect not supported (security restriction)".into(),
                    )
                    .into());
                }
                let ofs = env.ofs.clone();
                let ors = env.ors.clone();
                if exprs.is_empty() {
                    // print with no args prints $0
                    let s = env.get_field(0).to_str();
                    print!("{}{}", s, ors);
                } else {
                    let mut parts = Vec::new();
                    for e in exprs {
                        parts.push(self.eval_expr(e, env)?.to_str());
                    }
                    print!("{}{}", parts.join(&ofs), ors);
                }
                let _ = io::stdout().flush();
                Ok(ControlFlow::Normal)
            }
            Stmt::Printf(exprs, redir) => {
                if let Some(RedirTarget::File(_)) | Some(RedirTarget::Append(_)) = redir {
                    return Err(AwkError::Runtime(
                        "printf redirect not supported (security restriction)".into(),
                    )
                    .into());
                }
                if exprs.is_empty() {
                    return Ok(ControlFlow::Normal);
                }
                let fmt = self.eval_expr(&exprs[0], env)?.to_str();
                let mut args = Vec::new();
                for e in &exprs[1..] {
                    args.push(self.eval_expr(e, env)?);
                }
                let output = format_printf(&fmt, &args);
                print!("{}", output);
                let _ = io::stdout().flush();
                Ok(ControlFlow::Normal)
            }
            Stmt::Assign(name, op, val_expr) => {
                let current = env.get_var(name);
                let new_val = self.eval_expr(val_expr, env)?;
                let result = apply_assign_op(current, op, new_val)?;
                env.set_var(name, result);
                Ok(ControlFlow::Normal)
            }
            Stmt::FieldAssign(idx_expr, val_expr) => {
                let idx = self.eval_expr(idx_expr, env)?.to_num() as usize;
                let val = self.eval_expr(val_expr, env)?.to_str();
                env.set_field(idx, val);
                Ok(ControlFlow::Normal)
            }
            Stmt::ArrayAssign(arr, key_expr, op, val_expr) => {
                let key = self.eval_expr(key_expr, env)?.to_str();
                let current = env.get_array_val(arr, &key);
                let new_val = self.eval_expr(val_expr, env)?;
                let result = apply_assign_op(current, op, new_val)?;
                env.set_array_val(arr, &key, result);
                Ok(ControlFlow::Normal)
            }
            Stmt::If(cond, then_s, else_s) => {
                let cv = self.eval_expr(cond, env)?;
                if cv.is_truthy() {
                    self.exec_stmt(then_s, env)
                } else if let Some(else_stmt) = else_s {
                    self.exec_stmt(else_stmt, env)
                } else {
                    Ok(ControlFlow::Normal)
                }
            }
            Stmt::While(cond, body) => {
                loop {
                    let cv = self.eval_expr(cond, env)?;
                    if !cv.is_truthy() {
                        break;
                    }
                    match self.exec_stmt(body, env)? {
                        ControlFlow::Next => return Ok(ControlFlow::Next),
                        ControlFlow::Exit(c) => return Ok(ControlFlow::Exit(c)),
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Normal => {}
                    }
                }
                Ok(ControlFlow::Normal)
            }
            Stmt::Do(body, cond) => {
                loop {
                    match self.exec_stmt(body, env)? {
                        ControlFlow::Next => return Ok(ControlFlow::Next),
                        ControlFlow::Exit(c) => return Ok(ControlFlow::Exit(c)),
                        ControlFlow::Break => break,
                        ControlFlow::Continue => {}
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Normal => {}
                    }
                    let cv = self.eval_expr(cond, env)?;
                    if !cv.is_truthy() {
                        break;
                    }
                }
                Ok(ControlFlow::Normal)
            }
            Stmt::For(init, cond, incr, body) => {
                if let Some(init_stmt) = init {
                    self.exec_stmt(init_stmt, env)?;
                }
                loop {
                    if let Some(cond_expr) = cond {
                        let cv = self.eval_expr(cond_expr, env)?;
                        if !cv.is_truthy() {
                            break;
                        }
                    }
                    match self.exec_stmt(body, env)? {
                        ControlFlow::Next => return Ok(ControlFlow::Next),
                        ControlFlow::Exit(c) => return Ok(ControlFlow::Exit(c)),
                        ControlFlow::Break => break,
                        ControlFlow::Continue => {}
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Normal => {}
                    }
                    if let Some(incr_stmt) = incr {
                        self.exec_stmt(incr_stmt, env)?;
                    }
                }
                Ok(ControlFlow::Normal)
            }
            Stmt::ForIn(var, arr, body) => {
                let keys: Vec<String> = env
                    .arrays
                    .get(arr)
                    .map(|a| a.keys().cloned().collect())
                    .unwrap_or_default();
                for key in keys {
                    env.set_var(var, Value::Str(key));
                    match self.exec_stmt(body, env)? {
                        ControlFlow::Next => return Ok(ControlFlow::Next),
                        ControlFlow::Exit(c) => return Ok(ControlFlow::Exit(c)),
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Normal => {}
                    }
                }
                Ok(ControlFlow::Normal)
            }
            Stmt::Delete(arr, key_expr) => {
                if let Some(key_e) = key_expr {
                    let key = self.eval_expr(key_e, env)?.to_str();
                    if let Some(a) = env.arrays.get_mut(arr) {
                        a.remove(&key);
                    }
                } else {
                    env.arrays.remove(arr);
                }
                Ok(ControlFlow::Normal)
            }
            Stmt::Next => Ok(ControlFlow::Next),
            Stmt::Exit(code_expr) => {
                let code = if let Some(e) = code_expr {
                    self.eval_expr(e, env)?.to_num() as i32
                } else {
                    0
                };
                Ok(ControlFlow::Exit(code))
            }
            Stmt::Return(val_expr) => {
                let val = if let Some(e) = val_expr {
                    self.eval_expr(e, env)?
                } else {
                    Value::Uninitialized
                };
                Ok(ControlFlow::Return(val))
            }
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Expr(e) => {
                self.eval_expr(e, env)?;
                Ok(ControlFlow::Normal)
            }
        }
    }

    fn match_pattern(&self, pattern: &Pattern, rule_idx: usize, env: &mut Env) -> Result<bool> {
        match pattern {
            Pattern::Always => Ok(true),
            Pattern::Begin | Pattern::End => Ok(false),
            Pattern::Expr(e) => {
                let v = self.eval_expr(e, env)?;
                Ok(v.is_truthy())
            }
            Pattern::Range(start_e, end_e) => {
                let active = *env.range_active.get(&rule_idx).unwrap_or(&false);
                if active {
                    // Check if end pattern matches
                    let end_v = self.eval_expr(end_e, env)?;
                    if end_v.is_truthy() {
                        env.range_active.insert(rule_idx, false);
                    }
                    Ok(true)
                } else {
                    // Check if start pattern matches
                    let start_v = self.eval_expr(start_e, env)?;
                    if start_v.is_truthy() {
                        // Check end immediately too
                        let end_v = self.eval_expr(end_e, env)?;
                        if !end_v.is_truthy() {
                            env.range_active.insert(rule_idx, true);
                        }
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
            }
        }
    }

    fn run_rules(&self, rules: &[Rule], env: &mut Env, phase: &str) -> Result<ControlFlow> {
        for (idx, rule) in rules.iter().enumerate() {
            let matches = match phase {
                "begin" => matches!(rule.pattern, Pattern::Begin),
                "end" => matches!(rule.pattern, Pattern::End),
                "record" => self.match_pattern(&rule.pattern, idx, env)?,
                _ => false,
            };
            if matches {
                for stmt in &rule.action {
                    match self.exec_stmt(stmt, env)? {
                        ControlFlow::Next => return Ok(ControlFlow::Next),
                        ControlFlow::Exit(c) => return Ok(ControlFlow::Exit(c)),
                        cf @ ControlFlow::Return(_) => return Ok(cf),
                        _ => {}
                    }
                }
            }
        }
        Ok(ControlFlow::Normal)
    }
}

fn compare_values<F1, F2>(lv: &Value, rv: &Value, num_cmp: F1, str_cmp: F2) -> f64
where
    F1: Fn(f64, f64) -> bool,
    F2: Fn(&str, &str) -> bool,
{
    // If both look numeric, compare numerically; otherwise compare as strings
    let l_str = lv.to_str();
    let r_str = rv.to_str();
    let l_num = lv.to_num();
    let r_num = rv.to_num();

    // Use numeric comparison if both values are numeric
    let both_numeric = matches!(lv, Value::Num(_)) && matches!(rv, Value::Num(_))
        || (is_numeric_string(&l_str) && is_numeric_string(&r_str));

    if both_numeric {
        if num_cmp(l_num, r_num) { 1.0 } else { 0.0 }
    } else if str_cmp(&l_str, &r_str) {
        1.0
    } else {
        0.0
    }
}

fn is_numeric_string(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    s.parse::<f64>().is_ok()
}

fn apply_assign_op(current: Value, op: &AssignOp, new_val: Value) -> Result<Value> {
    match op {
        AssignOp::Set => Ok(new_val),
        AssignOp::Add => Ok(Value::Num(current.to_num() + new_val.to_num())),
        AssignOp::Sub => Ok(Value::Num(current.to_num() - new_val.to_num())),
        AssignOp::Mul => Ok(Value::Num(current.to_num() * new_val.to_num())),
        AssignOp::Div => {
            let d = new_val.to_num();
            if d == 0.0 {
                return Err(AwkError::Runtime("division by zero in assignment".into()).into());
            }
            Ok(Value::Num(current.to_num() / d))
        }
        AssignOp::Mod => {
            let d = new_val.to_num();
            if d == 0.0 {
                return Err(AwkError::Runtime("modulo by zero in assignment".into()).into());
            }
            Ok(Value::Num(current.to_num() % d))
        }
    }
}

// ============================================================
// Main execute function
// ============================================================

fn execute(program: &Program, args: &Args) -> Result<i32> {
    let mut env = Env::new(&args.field_sep, &args.assign)?;
    let interp = Interpreter::new(program);

    // Run BEGIN rules
    match interp.run_rules(&program.rules, &mut env, "begin")? {
        ControlFlow::Exit(c) => return Ok(c),
        _ => {}
    }

    // Process input files (or stdin)
    let files_empty = args.files.is_empty();
    let exit_code;

    if files_empty {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        exit_code = process_input(reader, "<stdin>", &mut env, &interp, program)?;
    } else {
        let mut code = 0;
        for path in &args.files {
            let path_str = gow_core::path::try_convert_msys_path(
                path.to_str().unwrap_or(""),
            );
            match File::open(Path::new(&path_str)) {
                Ok(f) => {
                    env.filename = path_str.clone();
                    env.fnr = 0;
                    let reader = BufReader::new(f);
                    code = process_input(reader, &path_str, &mut env, &interp, program)?;
                    if code != 0 {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("awk: {}: {}", path_str, e);
                    code = 2;
                    break;
                }
            }
        }
        exit_code = code;
    }

    if exit_code != 0 {
        return Ok(exit_code);
    }

    // Run END rules
    match interp.run_rules(&program.rules, &mut env, "end")? {
        ControlFlow::Exit(c) => return Ok(c),
        _ => {}
    }

    Ok(0)
}

fn process_input<R: BufRead>(
    reader: R,
    filename: &str,
    env: &mut Env,
    interp: &Interpreter<'_>,
    program: &Program,
) -> Result<i32> {
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("awk: {}: {}", filename, e);
                return Ok(2);
            }
        };

        env.nr += 1;
        env.fnr += 1;
        env.set_record(&line);

        match interp.run_rules(&program.rules, env, "record")? {
            ControlFlow::Exit(c) => return Ok(c),
            _ => {}
        }
    }
    Ok(0)
}

// ============================================================
// Public entry point
// ============================================================

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args_vec: Vec<OsString> = args.into_iter().collect();
    let parsed = match <Args as ClapParser>::try_parse_from(&args_vec) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return 2;
        }
    };

    // Get program text
    let program_text = if let Some(ref prog_file) = parsed.program_file {
        match std::fs::read_to_string(prog_file) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("awk: {}: {}", prog_file.display(), e);
                return 2;
            }
        }
    } else if let Some(ref prog) = parsed.program {
        prog.clone()
    } else {
        eprintln!("awk: no program specified");
        return 2;
    };

    // Lex
    let tokens = match lex(&program_text) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("awk: {}", e);
            return 2;
        }
    };

    // Parse
    let program = match parse(&tokens) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("awk: {}", e);
            return 2;
        }
    };

    // Execute
    match execute(&program, &parsed) {
        Ok(code) => code,
        Err(e) => {
            let msg = e.to_string();
            if let Some(code_str) = msg.strip_prefix("__exit__") {
                code_str.parse::<i32>().unwrap_or(1)
            } else {
                eprintln!("awk: {}", e);
                1
            }
        }
    }
}

// ============================================================
// Unit tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_basic() {
        let tokens = lex("{ print $1 }").unwrap();
        // Expected: LBrace, Print, Dollar, Number(1), RBrace, Eof
        assert!(tokens.iter().any(|t| matches!(t, Token::LBrace)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Print)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Dollar)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Number(n) if *n == 1.0)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RBrace)));
    }

    #[test]
    fn test_lex_string_literal() {
        let tokens = lex(r#"{ print "hello" }"#).unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::Str(s) if s == "hello")));
    }

    #[test]
    fn test_lex_regex() {
        let tokens = lex("/foo/ { print }").unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::Regex(s) if s == "foo")));
    }

    #[test]
    fn test_parse_always_pattern() {
        let tokens = lex("{ print $1 }").unwrap();
        let prog = parse(&tokens).unwrap();
        assert_eq!(prog.rules.len(), 1);
        assert!(matches!(prog.rules[0].pattern, Pattern::Always));
        assert!(!prog.rules[0].action.is_empty());
        // Action should contain a Print statement
        assert!(matches!(prog.rules[0].action[0], Stmt::Print(_, _)));
    }

    #[test]
    fn test_parse_begin_end() {
        let tokens = lex("BEGIN { x=0 } END { print x }").unwrap();
        let prog = parse(&tokens).unwrap();
        assert_eq!(prog.rules.len(), 2);
        assert!(matches!(prog.rules[0].pattern, Pattern::Begin));
        assert!(matches!(prog.rules[1].pattern, Pattern::End));
    }

    #[test]
    fn test_field_split_whitespace() {
        let fields = split_fields("  a  b  c  ", " ");
        // fields[0] = full record, fields[1..] = split parts
        assert_eq!(fields[0], "  a  b  c  ");
        assert_eq!(fields[1], "a");
        assert_eq!(fields[2], "b");
        assert_eq!(fields[3], "c");
        assert_eq!(fields.len(), 4);
    }

    #[test]
    fn test_field_split_single_char() {
        let fields = split_fields("a:b:c", ":");
        assert_eq!(fields[0], "a:b:c");
        assert_eq!(fields[1], "a");
        assert_eq!(fields[2], "b");
        assert_eq!(fields[3], "c");
    }

    #[test]
    fn test_field_split_regex() {
        let fields = split_fields("a::b::c", "::");
        assert_eq!(fields[0], "a::b::c");
        assert_eq!(fields[1], "a");
        assert_eq!(fields[2], "b");
        assert_eq!(fields[3], "c");
    }

    #[test]
    fn test_value_to_num() {
        assert_eq!(Value::Num(3.14).to_num(), 3.14);
        assert_eq!(Value::Str("42".to_string()).to_num(), 42.0);
        assert_eq!(Value::Str("  10abc".to_string()).to_num(), 10.0);
        assert_eq!(Value::Uninitialized.to_num(), 0.0);
    }

    #[test]
    fn test_value_to_str() {
        assert_eq!(Value::Num(42.0).to_str(), "42");
        assert_eq!(Value::Num(3.14).to_str(), "3.14");
        assert_eq!(Value::Str("hello".to_string()).to_str(), "hello");
        assert_eq!(Value::Uninitialized.to_str(), "");
    }

    #[test]
    fn test_format_printf_d() {
        let result = format_printf("%05d", &[Value::Num(5.0)]);
        assert_eq!(result, "00005");
    }

    #[test]
    fn test_format_printf_s() {
        let result = format_printf("%s", &[Value::Str("hello".into())]);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_printf_f() {
        let result = format_printf("%.2f", &[Value::Num(3.14159)]);
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_format_printf_percent() {
        let result = format_printf("100%%", &[]);
        assert_eq!(result, "100%");
    }
}
