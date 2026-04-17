use postfix_rs::rewrite::step;


#[derive(Debug, Clone)]
pub enum AppError {
    TooFewArguments
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::TooFewArguments => write!(f, "too few arguments")
        }
    }
}
impl std::error::Error for AppError {}


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args: Vec<String> = std::env::args().collect();

    let mut program = args.get(1).ok_or(AppError::TooFewArguments)?.to_owned();
    let mut args = args.get(2).ok_or(AppError::TooFewArguments)?.to_owned();

    loop {
        println!("\"{}\" \"{}\"", program, args);
        let (new_program, new_args) = step(&program, &args)?;
        if program == new_program {
            break;
        }
        else {
            program = new_program;
            args = new_args;
        }
    }

    /*
    let result = eval(argc, postfix_args, command_sequence)?;

    println!("result: {}", result);
    */

    Ok(())
}


#[cfg(test)]
mod tests {
    use std::num::ParseIntError;
    use postfix_rs::postfix::PostfixError::{self, *};

    use postfix_rs::parser::{lexer, parse};
    use postfix_rs::postfix::eval;

    fn parse_and_eval(input: &str, args: &str) -> Result<isize, PostfixError> {
        let mut tokens = lexer(input).unwrap();
        let postfix_args = args
            .split_whitespace()
            .map(|s| s.parse::<isize>())
            .collect::<Result<Vec<isize>, ParseIntError>>().unwrap();

        let (argc, command_sequence) = parse(&mut tokens).unwrap();
        eval(argc, postfix_args, command_sequence)
    }

    #[test]
    fn test_basic_stack_ops() {
        let test_cases = [
            ("(postfix 0 1 2 3)", "", Ok(3)),
            ("(postfix 0 1 2 3 pop)", "", Ok(2)),
            ("(postfix 0 1 2 swap 3 pop)", "", Ok(1)),
            ("(postfix 0 1 swap)", "", Err(StackEmpty)),
            ("(postfix 0 1 pop pop)", "", Err(StackEmpty)),
        ];

        for (command_sequence, args, expect) in test_cases {
            assert_eq!(parse_and_eval(command_sequence, args), expect);
        }
    }

    #[test]
    fn test_args() {
        let test_cases = [
            ("(postfix 2)", "3 4", Ok(3)),
            ("(postfix 2 swap)", "3 4", Ok(4)),
            ("(postfix 3 pop swap)", "3 4 5", Ok(5)),
            ("(postfix 2 swap)", "3", Err(WrongNumberOfArguments)),
            ("(postfix 1 pop)", "4 5", Err(WrongNumberOfArguments)),
        ];

        for (command_sequence, args, expect) in test_cases {
            assert_eq!(parse_and_eval(command_sequence, args), expect);
        }
    }
}