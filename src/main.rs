use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
enum Command {
    Number(isize),
    BinaryOp(BinaryOp),
    Swap,
    Pop,
    Sel,
    Nget,
    ExecutableSequence(Vec<Command>),
    Exec
}

#[derive(Clone, Debug, PartialEq)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    LT,
    GT,
    EQ,
}


#[derive(Clone, Debug, PartialEq)]
enum PostfixError {
    WrongNumberOfArguments,
    StackEmpty,
    InvalidFinalStackTop,
    OperandNotAnInteger,
    DivideByZero,
    IndexOutOfBounds,
    NotAnExecutableSequence
}


impl std::fmt::Display for PostfixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostfixError::WrongNumberOfArguments => write!(f, "wrong number of arguments"),
            PostfixError::StackEmpty => write!(f, "stack is empty"),
            PostfixError::InvalidFinalStackTop => write!(f, "invalid final stack top"),
            PostfixError::OperandNotAnInteger => write!(f, "operand is not an integer"),
            PostfixError::DivideByZero => write!(f, "divide by zero"),
            PostfixError::IndexOutOfBounds => write!(f, "index out of bounds"),
            PostfixError::NotAnExecutableSequence => write!(f, "not an executable sequence")
        }
    }
}


impl std::error::Error for PostfixError {}


fn eval(argc: usize, args: Vec<isize>, commands: Vec<Command>) -> Result<isize, PostfixError> {
    if argc != args.len() {
        return Err(PostfixError::WrongNumberOfArguments)
    }
    let mut stack = Vec::<Command>::new();
    stack.extend(args.iter().rev().map(|n|Command::Number(*n)));

    let mut commands = VecDeque::from(commands);

    while let Some(command) = commands.pop_front() {
        match command {
            c @ Command::Number(_) => stack.push(c.clone()),
            c @ Command::ExecutableSequence(_) => stack.push(c.clone()),
            Command::BinaryOp(op) => {
                let v1 = stack.pop()
                    .ok_or(PostfixError::StackEmpty)
                    .and_then(|c| match c {
                        Command::Number(n) => Ok(n),
                        _ => Err(PostfixError::OperandNotAnInteger)
                    })?;
                let v2 = stack.pop().ok_or(PostfixError::StackEmpty)
                    .and_then(|c| match c {
                        Command::Number(n) => Ok(n),
                        _ => Err(PostfixError::OperandNotAnInteger)
                    })?;
                match op {
                    BinaryOp::Add => stack.push(Command::Number(v2 + v1)),
                    BinaryOp::Sub => stack.push(Command::Number(v2 - v1)),
                    BinaryOp::Mul => stack.push(Command::Number(v2 * v1)),
                    BinaryOp::Div => {
                        if v1 == 0 {
                            return Err(PostfixError::DivideByZero)
                        }
                        stack.push(Command::Number(v2 / v1))
                    },
                    BinaryOp::Rem => {
                        if v1 == 0 {
                            return Err(PostfixError::DivideByZero)
                        }
                        stack.push(Command::Number(v2 % v1))
                    },
                    BinaryOp::LT => stack.push(Command::Number((v2 < v1) as isize)),
                    BinaryOp::EQ => stack.push(Command::Number((v2 == v1) as isize)),
                    BinaryOp::GT => stack.push(Command::Number((v2 > v1) as isize)),
                }
            },
            Command::Swap => {
                let v1 = stack.pop().ok_or(PostfixError::StackEmpty)?;
                let v2 = stack.pop().ok_or(PostfixError::StackEmpty)?;
                stack.push(v1);
                stack.push(v2);
            },
            Command::Pop => _ = stack.pop().ok_or(PostfixError::StackEmpty)?,
            Command::Sel => {
                let v1 = stack.pop().ok_or(PostfixError::StackEmpty)?;
                let v2 = stack.pop().ok_or(PostfixError::StackEmpty)?;
                let v3 = stack.pop().ok_or(PostfixError::StackEmpty)
                    .and_then(|c| match c {
                        Command::Number(n) => Ok(n),
                        _ => Err(PostfixError::OperandNotAnInteger)
                    })?;
                stack.push(if v3 == 0 { v1 } else { v2 });
            }
            Command::Nget => {
                let i = stack.pop().ok_or(PostfixError::StackEmpty)
                    .and_then(|c| match c {
                        Command::Number(n) => Ok(n),
                        _ => Err(PostfixError::OperandNotAnInteger)
                    })?;
                let v_index = stack
                    .iter()
                    .rev()
                    .nth(
                        (i-1).try_into().or(Err(PostfixError::IndexOutOfBounds))?
                    )
                    .ok_or(PostfixError::IndexOutOfBounds)
                    .and_then(|c| match c {
                        Command::Number(n) => Ok(*n),
                        _ => Err(PostfixError::OperandNotAnInteger)
                    })?;
                stack.push(Command::Number(v_index));
            }
            Command::Exec => {
                let executable_sequence = stack.pop().ok_or(PostfixError::StackEmpty)
                    .and_then(|c| match c {
                        Command::ExecutableSequence(e) => Ok(e),
                        _ => Err(PostfixError::NotAnExecutableSequence)
                    })?;
                for c in executable_sequence.into_iter().rev() {
                    commands.push_front(c);
                }
            }
        }
    }

    stack.pop()
        .ok_or(PostfixError::StackEmpty)
        .and_then(|c| match c {
            Command::Number(n) => Ok(n),
            _ => Err(PostfixError::InvalidFinalStackTop)
        })
}


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let command_sequence = vec![
        Command::ExecutableSequence(vec![
            Command::Number(7),
            Command::Swap,
            Command::Exec
        ]),
        Command::ExecutableSequence(vec![
            Command::Number(0),
            Command::Swap,
            Command::BinaryOp(BinaryOp::Sub)
        ]),
        Command::Swap,
        Command::Exec
    ];

    let result = eval(0, vec![], command_sequence)?;

    println!("result: {}", result);

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use Command::*;
    use super::BinaryOp::*;

    #[test]
    fn test_basic_stack_ops() {
        let test_cases = [
            (
                0,
                vec![Number(1), Number(2), Number(3)],
                Ok(3)
            ),
            (
                0,
                vec![Number(1), Number(2), Number(3), Pop],
                Ok(2)
            ),
            (
                0,
                vec![Number(1), Number(2), Swap, Number(3), Pop],
                Ok(1)
            ),
            (
                0,
                vec![Number(1), Swap],
                Err(PostfixError::StackEmpty)
            ),
            (
                0,
                vec![Number(1), Pop, Pop],
                Err(PostfixError::StackEmpty)
            )
        ];

        for (argc, test_case, expect) in test_cases {
            assert_eq!(eval(argc, vec![], test_case), expect);
        }
    }

    #[test]
    fn test_args() {
        let test_cases = vec![
            (
                2,
                vec![],
                vec![3, 4],
                Ok(3)
            ),
            (
                2,
                vec![Swap],
                vec![3, 4],
                Ok(4)
            ),
            (
                3,
                vec![Pop, Swap],
                vec![3, 4, 5],
                Ok(5)
            )
        ];

        for(argc, test_case, args, expect) in test_cases {
            assert_eq!(eval(argc, args, test_case), expect)
        }
    }

    #[test]
    fn test_semantics_operations() {
        let test_cases = vec![
            // pp.11 (Semantics 1.4.2)
            (
                2,
                vec![Swap],
                vec![3],
                Err(PostfixError::WrongNumberOfArguments)
            ),
            (
                1,
                vec![Pop],
                vec![4, 5],
                Err(PostfixError::WrongNumberOfArguments)
            ),
            (
                1,
                vec![Number(4), BinaryOp(Sub)],
                vec![3],
                Ok(-1)
            ),
            (
                1,
                vec![Number(4), BinaryOp(Add), Number(5), BinaryOp(Mul), Number(6), BinaryOp(Sub), Number(7), BinaryOp(Div)],
                vec![3],
                Ok(4)
            ),
            (
                5,
                vec![BinaryOp(Add), BinaryOp(Mul), BinaryOp(Sub), Swap, BinaryOp(Div)],
                vec![7, 6, 5, 4, 3],
                Ok(-20)
            ),
            (
                3,
                vec![Number(4000), Swap, Pop, BinaryOp(Add)],
                vec![300, 20, 1],
                Ok(4020)
            ),
            (
                2,
                vec![BinaryOp(Add), Number(2), BinaryOp(Div)],
                vec![3, 7],
                Ok(5)
            ),
            (
                1,
                vec![Number(3), BinaryOp(Div)],
                vec![17],
                Ok(5)
            ),
            (
                1,
                vec![Number(3), BinaryOp(Rem)],
                vec![17],
                Ok(2)
            ),
            (
                1,
                vec![Number(4), BinaryOp(LT)],
                vec![3],
                Ok(1)
            ),
            (
                1,
                vec![Number(4), BinaryOp(LT)],
                vec![5],
                Ok(0)
            ),
            (
                1,
                vec![Number(4), BinaryOp(LT), Number(10), BinaryOp(Add)],
                vec![3],
                Ok(11)
            ),
            (
                1,
                vec![Number(4), BinaryOp(Mul), BinaryOp(Add)],
                vec![3],
                Err(PostfixError::StackEmpty) // Not enough numbers to add.
            ),
            (
                2,
                vec![Number(4), BinaryOp(Sub), BinaryOp(Div)],
                vec![4, 5],
                Err(PostfixError::DivideByZero)
            )
        ];

        for (argc, commands, args, expect) in test_cases {
            assert_eq!(eval(argc, args, commands), expect);
        }
    }

    #[test]
    fn test_nget() {
        let test_cases = vec![
            // pp.12 (nget の基本動作)
            (
                2,
                vec![Number(1), Nget],
                vec![4, 5],
                Ok(4)
            ),
            (
                2,
                vec![Number(2), Nget],
                vec![4, 5],
                Ok(5)
            ),
            // pp.12 (nget のエラーケース)
            (
                2,
                vec![Number(3), Nget],
                vec![4, 5],
                Err(PostfixError::IndexOutOfBounds) // Index 3 is too large.
            ),
            (
                2,
                vec![Number(0), Nget],
                vec![4, 5],
                Err(PostfixError::IndexOutOfBounds) // Index 0 is too small.
            ),
            (
                1,
                vec![ExecutableSequence(vec![Number(2), BinaryOp(Mul)]), Number(1), Nget],
                vec![3],
                Err(PostfixError::OperandNotAnInteger) // Value at index 1 is an executable sequence.
            ),
            // pp.12 (nget の応用例)
            (
                1,
                vec![Number(1), Nget, BinaryOp(Mul)],
                vec![5],
                Ok(25) // A squaring program.
            ),
            (
                4,
                vec![
                    Number(4), Nget, Number(5), Nget, BinaryOp(Mul), BinaryOp(Mul),
                    Swap, Number(4), Nget, BinaryOp(Mul), BinaryOp(Add), BinaryOp(Add)
                ],
                vec![3, 4, 5, 2],
                Ok(25) // Calculates ax^2 + bx + c
            )
        ];

        for (argc, commands, args, expect) in test_cases {
            assert_eq!(eval(argc, args, commands), expect);
        }
    }

    #[test]
    fn test_exec_and_sel() {
        let test_cases = vec![
            // pp.13 (exec の例: サブルーチン的な動作)
            (
                1,
                vec![ExecutableSequence(vec![Number(2), BinaryOp(Mul)]), Exec],
                vec![7],
                Ok(14)
            ),
            (
                0,
                vec![ExecutableSequence(vec![Number(0), Swap, BinaryOp(Sub)]), Number(7), Swap, Exec],
                vec![],
                Ok(-7)
            ),
            // pp.13 (exec 関連のエラーケース)
            (
                0,
                vec![ExecutableSequence(vec![Number(2), BinaryOp(Mul)])],
                vec![],
                Err(PostfixError::InvalidFinalStackTop)
            ),
            (
                0,
                vec![Number(3), ExecutableSequence(vec![Number(2), BinaryOp(Mul)]), BinaryOp(GT)],
                vec![],
                Err(PostfixError::OperandNotAnInteger)
            ),
            (
                0,
                vec![Number(3), Exec],
                vec![],
                Err(PostfixError::NotAnExecutableSequence)
            ),
            // pp.13 (複雑な exec の組み合わせ)
            (
                0,
                vec![
                    ExecutableSequence(vec![Number(7), Swap, Exec]),
                    ExecutableSequence(vec![Number(0), Swap, BinaryOp(Sub)]), Swap, Exec
                ],
                vec![],
                Ok(-7)
            ),
            (
                2,
                vec![
                    ExecutableSequence(vec![BinaryOp(Mul), BinaryOp(Sub)]),
                    ExecutableSequence(vec![Number(1), Nget, BinaryOp(Mul)]),
                    Number(4), Nget, Swap, Exec, Swap, Exec
                ],
                vec![-10, 2],
                Ok(42) // Calculates b - a * b^2
            ),
            // pp.13 (sel の例: 条件選択)
            (
                1,
                vec![Number(2), Number(3), Sel],
                vec![1],
                Ok(2)
            ),
            (
                1,
                vec![Number(2), Number(3), Sel],
                vec![0],
                Ok(3)
            ),
            (
                1,
                vec![Number(2), Number(3), Sel],
                vec![17],
                Ok(2) // Any nonzero number is "true."
            ),
            (
                0,
                vec![ExecutableSequence(vec![Number(2), BinaryOp(Mul)]), Number(3), Number(4), Sel],
                vec![],
                Err(PostfixError::OperandNotAnInteger) // Test not a number.
            ),
            // pp.13 (sel と exec の組み合わせ)
            (
                4,
                vec![
                    BinaryOp(LT), ExecutableSequence(vec![BinaryOp(Add)]),
                    ExecutableSequence(vec![BinaryOp(Mul)]), Sel, Exec
                ],
                vec![3, 4, 5, 6],
                Ok(30)
            ),
            (
                4,
                vec![
                    BinaryOp(LT), ExecutableSequence(vec![BinaryOp(Add)]),
                    ExecutableSequence(vec![BinaryOp(Mul)]), Sel, Exec
                ],
                vec![4, 3, 5, 6],
                Ok(11)
            ),
            // pp.13 (絶対値プログラム)
            (
                1,
                vec![
                    Number(1), Nget, Number(0), BinaryOp(LT),
                    ExecutableSequence(vec![Number(0), Swap, BinaryOp(Sub)]),
                    ExecutableSequence(vec![]), Sel, Exec
                ],
                vec![-7],
                Ok(7)
            ),
            (
                1,
                vec![
                    Number(1), Nget, Number(0), BinaryOp(LT),
                    ExecutableSequence(vec![Number(0), Swap, BinaryOp(Sub)]),
                    ExecutableSequence(vec![]), Sel, Exec
                ],
                vec![6],
                Ok(6)
            )
        ];

        for (argc, commands, args, expect) in test_cases {
            assert_eq!(eval(argc, args, commands), expect);
        }
    }
}