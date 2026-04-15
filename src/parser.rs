use std::{collections::VecDeque, iter::successors};
use crate::postfix::{Command, BinaryOp};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Postfix,
    Number(isize),
    Command(String),
    EOF
}


#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    InvalidCharacter,
    UnknownCommand,
    UnexpectedToken,
    UnexpectedEOF,
    InvalidNumber,
    InvalidSyntax,
    InvalidArgumentCount
}


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidCharacter => write!(f, "invalid character"),
            ParseError::UnknownCommand => write!(f, "unknown command"),
            ParseError::UnexpectedToken => write!(f, "unexpected token"),
            ParseError::UnexpectedEOF => write!(f, "unexpected EOF"),
            ParseError::InvalidNumber => write!(f, "invalid number"),
            ParseError::InvalidSyntax => write!(f, "invalid syntax"),
            ParseError::InvalidArgumentCount => write!(f, "invalid argument count"),
        }
    }
}


impl std::error::Error for ParseError {}


pub fn lexer(input: &str) -> Result<VecDeque<Token>, ParseError> {
    use self::Token::*;
    use self::ParseError::*;

    let mut result = VecDeque::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '(' => {
                result.push_back(LParen);
            },
            ')' => {
                result.push_back(RParen);
            },
            first @ 'a'..='z' => {
                let identifier = successors(Some(first), |_|{
                    chars.next_if(|c|c.is_ascii_lowercase())
                })
                .collect();
                
                result.push_back(if identifier == "postfix" { Postfix } else { Command(identifier) })
            },
            first @ ('0'..='9' | '-') => {
                let digits = successors(Some(first), |_|{
                    chars.next_if(|c|c.is_ascii_digit())
                })
                .collect::<String>();

                result.push_back(
                    Number(digits.parse().map_err(|_| InvalidNumber)?)
                );
            },
            ' ' => {
                continue;
            }
            _ => return Err(InvalidCharacter)
        }
    }
    result.push_back(EOF);
    Ok(result)
}


pub fn parse(input: &mut VecDeque<Token>) -> Result<(usize, Vec<Command>), ParseError> {
    use self::Token::*;
    use self::ParseError::*;

    expect(input, LParen)?;
    expect(input, Postfix)?;

    let argc = input.pop_front().ok_or(InvalidSyntax).and_then(|t| {
        if let Number(n) = t {
            if n >= 0 {
                Ok(n)
            }
            else {
                Err(InvalidArgumentCount)
            }
        }
        else {
            Err(InvalidSyntax)
        } 
    })?;

    let result = parse_rec(input)?;

    expect(input, EOF)?;

    Ok((argc as usize, result))
}


fn expect(input: &mut VecDeque<Token>, token: Token) -> Result<(), ParseError>{
    use self::ParseError::*;

    input.pop_front().ok_or(InvalidSyntax).and_then(|t| {
        if t == token { Ok(()) } else {Err(UnexpectedToken)} 
    })
} 

fn parse_rec(input: &mut VecDeque<Token>) -> Result<Vec<Command>, ParseError> {
    use self::ParseError::*;

    let mut result = Vec::new();

    while let Some(token) = input.pop_front() {
        match token {
            Token::LParen => {
                result.push(Command::ExecutableSequence(parse_rec(input)?));
            },
            Token::Number(n) => result.push(Command::Number(n)),
            Token::RParen => return Ok(result),
            Token::Postfix => return Err(UnexpectedToken),
            Token::Command(s) => {
                match s.as_str() {
                    "swap" => result.push(Command::Swap),
                    "pop" => result.push(Command::Pop),
                    "nget" => result.push(Command::Nget),
                    "exec" => result.push(Command::Exec),
                    "sel" => result.push(Command::Sel),
                    "add" => result.push(Command::BinaryOp(BinaryOp::Add)),
                    "sub" => result.push(Command::BinaryOp(BinaryOp::Sub)),
                    "mul" => result.push(Command::BinaryOp(BinaryOp::Mul)),
                    "div" => result.push(Command::BinaryOp(BinaryOp::Div)),
                    "rem" => result.push(Command::BinaryOp(BinaryOp::Rem)),
                    "lt" => result.push(Command::BinaryOp(BinaryOp::LT)),
                    "gt" => result.push(Command::BinaryOp(BinaryOp::GT)),
                    "eq" => result.push(Command::BinaryOp(BinaryOp::EQ)),
                    _ => return Err(UnknownCommand)
                }
            },
            Token::EOF => return Err(UnexpectedEOF),
        }
    }

    Ok(result)
}