use std::error::Error;


#[derive(Clone, Debug)]
pub struct PostfixError{}

impl std::fmt::Display for PostfixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "error occured!")
    }
}

impl Error for PostfixError {}


trait StrExt {
    fn skip_while<P: Fn(char) -> bool>(&self, predicate: P) -> &Self;
    fn split_prefix_pattern<'a, P: Fn(&str) -> &str>(&'a self, pat: P) -> Option<(&'a str, &'a str)>;
    fn skip_pattern<'a, P: Fn(&str) -> &str>(&'a self, pat: P) -> &'a str;
}


impl StrExt for str {
    fn skip_while<P: Fn(char) -> bool>(&self, predicate: P) -> &Self {
        let mut chars = self.chars();
        let mut result = self;
        while let Some(c) = chars.next() {
            if predicate(c) {
                result = &result[c.len_utf8()..];
            }
            else {
                break;
            }
        }
        result
    }

    fn split_prefix_pattern<'a, P: Fn(&str) -> &str>(&'a self, pat: P) -> Option<(&'a str, &'a str)> {
        let rest = pat(self);
        if self.as_ptr() == rest.as_ptr() {
            None
        }
        else {
            Some((&self[..(rest.as_ptr() as usize - self.as_ptr() as usize)], rest))
        }
    }

    fn skip_pattern<'a, P: Fn(&str) -> &str>(&'a self, pat: P) -> &'a str {
        pat(self)
    }
}

fn skip_whitespace<'a>(input: &'a str) -> &'a str {
    input.skip_while(|c| c.is_whitespace())
}

fn skip_number<'a>(input: &'a str) -> &'a str {
    if input.starts_with('0') {
        &input['0'.len_utf8()..]
    }
    else if input.starts_with(|c| ('1'..'9').contains(&c) || c == '-') {
        input[1..].skip_while(|c| c.is_numeric())
    }
    else {
        input
    }
}

fn skip_paren<'a>(input: &'a str) -> &'a str {
    if input.starts_with('(') {
        let mut cursor = 0;
        let mut depth = 0;
        for c in input.chars() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => (),
            }
            cursor += c.len_utf8();

            if depth == 0 {
                return &input[cursor..];
            }
        }
        input
    }
    else {
        input
    }
}

fn skip_identifier<'a>(input: &'a str) -> &'a str {
    if input.starts_with(|c: char| c.is_alphabetic()) {
        input.skip_while(|c| c.is_alphabetic())
    }
    else {
        input
    }
}

pub fn step(input: &str, args: &str) -> Result<(String, String), PostfixError> {
    let (argc, mut rest) = input.trim_start_matches("(postfix")
        .skip_pattern(skip_whitespace)
        .split_prefix_pattern(skip_number)
        .ok_or(PostfixError {})
        .and_then(|(argc, rest)|{
            argc.parse::<usize>()
                .and_then(|n| Ok((n, rest)))
                .or_else(|_| Err(PostfixError{}))
        })?;
    
    let argv = args
        .split_whitespace()
        .map(|s| s.parse::<isize>())
        .collect::<Result<Vec<_>, _>>()
        .or_else(|_| Err(PostfixError{}))?;
    
    if argv.len() != argc {
        return Err(PostfixError {})
    }

    if argc > 0 {
        let args = argv.iter().rev()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        return Ok((format!("(postfix 0 {}{}", args, rest), String::new()))
    }

    let mut commands = Vec::<&str>::new();

    while let Some((com, r)) = rest.split_prefix_pattern(skip_whitespace)
        .or_else(|| rest.split_prefix_pattern(skip_number))
        .or_else(|| rest.split_prefix_pattern(skip_paren))
    {
        if !com.chars().all(|c| c.is_whitespace()) {
            commands.push(com);
        }
        rest = r;
    }

    if !rest.ends_with(")") {
        return Err(PostfixError{});
    }

    if let Some((op, _)) = rest.split_prefix_pattern(skip_identifier) {
        let (result, range) = match op {
            "add" | "sub" | "mul" | "div" |
            "rem" | "lt"  | "eq"  | "gt"  |
            "swap" => {
                let v1 = *commands.last().ok_or(PostfixError{})?;
                let v2 = *commands.get(
                        commands.len().checked_sub(2).ok_or(PostfixError{})?
                    ).ok_or(PostfixError{})?;
                
                let replace_start_ptr = v2.as_ptr() as usize;
                let replace_end_ptr = op.as_ptr() as usize + op.len();
                let input_start_ptr = input.as_ptr() as usize;

                let replace_start = replace_start_ptr - input_start_ptr;
                let replace_end = replace_end_ptr - input_start_ptr;

                let range = replace_start..replace_end;
                
                (binary_op(op, v1, v2)?, range)
            },
            "pop" | "exec" | "nget" => {
                let v = *commands.last().ok_or(PostfixError{})?;
                
                let replace_start_ptr = v.as_ptr() as usize;
                let replace_end_ptr = op.as_ptr() as usize + op.len();
                let input_start_ptr = input.as_ptr() as usize;

                let replace_start = replace_start_ptr - input_start_ptr;
                let replace_end = replace_end_ptr - input_start_ptr;

                let range = replace_start..replace_end;

                (unary_op(op, v, &commands)?, range)
            }
            "sel" => {
                let v1 = *commands.last().ok_or(PostfixError{})?;
                let v2 = *commands.get(
                        commands.len().checked_sub(2).ok_or(PostfixError{})?
                    ).ok_or(PostfixError{})?;
                let v3 = commands.get(
                        commands.len().checked_sub(3).ok_or(PostfixError{})?
                    ).ok_or(PostfixError{})?;

                let replace_start_ptr = v3.as_ptr() as usize;
                let replace_end_ptr = op.as_ptr() as usize + op.len();
                let input_start_ptr = input.as_ptr() as usize;

                let replace_start = replace_start_ptr - input_start_ptr;
                let replace_end = replace_end_ptr - input_start_ptr;

                let range = replace_start..replace_end;

                let v3 = v3.parse::<isize>().or(Err(PostfixError{}))?;
                if v3 == 0 {
                    (v1.to_string(), range)
                }
                else {
                    (v2.to_string(), range)
                }
            }
            _ => return Err(PostfixError{})
        };

        let mut res = input.to_string();
        res.replace_range(range, &result);
        Ok((res, String::new()))
    }
    else {
        Ok((input.to_string(), args.to_string()))
    }
}

fn unary_op(op: &str, v: &str, commands: &[&str]) -> Result<String, PostfixError> {
    match op {
        "pop" => Ok(String::new()),
        "exec" => {
            if v.starts_with('(') &&v.ends_with(')') {
                Ok(v['('.len_utf8()..(v.len() - ')'.len_utf8())].trim().to_string())
            }
            else {
                Err(PostfixError{})
            }
        },
        "nget" => {
            let index = v.parse::<isize>().or(Err(PostfixError{}))?;
            commands.get(
                commands.len().checked_sub_signed(index + 1).ok_or(PostfixError{})?
            )
            .ok_or(PostfixError{})
            .and_then(|s| Ok(s.to_string()))
        }
        _ => Err(PostfixError{})
    }
}

fn binary_op(op: &str, v1: &str, v2: &str) -> Result<String, PostfixError> {
    if op == "swap" {
        Ok([v1, v2].join(" "))
    }
    else {
        let v1 = v1.parse::<isize>()
            .or_else(|_| Err(PostfixError{}))?;
        let v2 = v2.parse::<isize>()
            .or_else(|_| Err(PostfixError{}))?;
        let result = match op {
            "add" => v2 + v1,
            "sub" => v2 - v1,
            "mul" => v2 * v1,
            "div" => v2.checked_div(v1).ok_or(PostfixError{})?,
            "rem" => v2.checked_rem(v1).ok_or(PostfixError{})?,
            "lt" => (v2 < v1) as isize,
            "gt" => (v2 > v1) as isize,
            "eq" => (v2 + v1) as isize,
            _ => return Err(PostfixError{})
        };
        Ok(result.to_string())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_while() {
        assert_eq!(
            "   string is here!".skip_while(
                |c: char| c.is_whitespace()
            ),
            "string is here!"
        );

        assert_eq!(
            "12345numbers are skipped!".skip_while(
                |c: char| c.is_numeric()
            ),
            "numbers are skipped!"
        );

        assert_eq!(
            "skip until end of slice".skip_while(
                |_| true
            ),
            ""
        );

        assert_eq!(
            "none is skipped!".skip_while(
                |_| false
            ),
            "none is skipped!"
        )
    }

    #[test]
    fn test_split_prefix_pattern() {
        assert_eq!(
            "1234rest".split_prefix_pattern(skip_number),
            Some(("1234", "rest"))
        );

        assert_eq!(
            "(this (parenthesized) part is skipped)this part is not skipped".split_prefix_pattern(skip_paren),
            Some(("(this (parenthesized) part is skipped)", "this part is not skipped"))
        );

        assert_eq!(
            "01234 is not recognized as a number".split_prefix_pattern(skip_number),
            None
        );
    }
}