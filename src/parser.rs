use std::str::Chars;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    IncrementPointer,
    DecrementPointer,
    IncrementData,
    DecrementData,
    Output,
    Input,
    Loop { body: Vec<Command> },
}

pub fn parse(input: &str) -> Result<Vec<Command>, Error> {
    let (commands, _) = parse_commands(input.chars(), false)?;
    Ok(commands)
}

fn parse_commands(mut input: Chars, mut in_loop: bool) -> Result<(Vec<Command>, Chars), Error> {
    let mut output = Vec::new();
    while let Some(token) = input.next() {
        let command = match token {
            '>' => Some(Command::IncrementPointer),
            '<' => Some(Command::DecrementPointer),
            '+' => Some(Command::IncrementData),
            '-' => Some(Command::DecrementData),
            ',' => Some(Command::Input),
            '.' => Some(Command::Output),
            '[' => {
                let (body, next_input) = parse_commands(input, true)?;
                input = next_input;
                Some(Command::Loop { body })
            }
            ']' => {
                if !in_loop {
                    return Err(Error::BrokenLoop);
                } else {
                    in_loop = false;
                    break;
                }
            }
            _ => None,
        };
        if let Some(command) = command {
            output.push(command);
        }
    }
    if !in_loop {
        Ok((output, input))
    } else {
        Err(Error::BrokenLoop)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Broken loop")]
    BrokenLoop,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_valid(input: &str) -> Vec<Command> {
        parse(input).expect("Invalid input")
    }

    fn parse_invalid(input: &str) -> Error {
        parse(input).unwrap_err()
    }

    #[test]
    fn increment_pointer() {
        assert_eq!(vec![Command::IncrementPointer], parse_valid(">"));
    }

    #[test]
    fn decrement_pointer() {
        assert_eq!(vec![Command::DecrementPointer], parse_valid("<"));
    }

    #[test]
    fn increment_data() {
        assert_eq!(vec![Command::IncrementData], parse_valid("+"));
    }

    #[test]
    fn decrement_data() {
        assert_eq!(vec![Command::DecrementData], parse_valid("-"));
    }

    #[test]
    fn input() {
        assert_eq!(vec![Command::Input], parse_valid(","));
    }

    #[test]
    fn output() {
        assert_eq!(vec![Command::Output], parse_valid("."));
    }

    #[test]
    fn single_loop() {
        let program = vec![Command::Loop {
            body: vec![Command::Input, Command::IncrementData],
        }];
        assert_eq!(program, parse_valid("[,+]"));
    }

    #[test]
    fn nested_loop() {
        let program = vec![Command::Loop {
            body: vec![Command::Loop {
                body: vec![Command::Input],
            }],
        }];
        assert_eq!(program, parse_valid("[[,]]"));
    }

    #[test]
    fn unfinished_loop() {
        assert!(matches!(parse_invalid("["), Error::BrokenLoop));
    }

    #[test]
    fn unfinished_nested_loop() {
        assert!(matches!(parse_invalid("[["), Error::BrokenLoop));
    }

    #[test]
    fn unopen_loop() {
        assert!(matches!(parse_invalid("]"), Error::BrokenLoop));
    }

    #[test]
    fn comment() {
        assert_eq!(Vec::<Command>::new(), parse_valid("potato"));
    }

    #[test]
    fn multi_line_comment() {
        let input = r#"to potato
        or not to potato"#;
        assert_eq!(Vec::<Command>::new(), parse_valid(input));
    }

    #[test]
    fn comment_within_code() {
        let input = r#"+.[This is part of the loop
        I can also increment the pointer >
        and print some stuff .
        ]"#;
        assert_eq!(
            vec![
                Command::IncrementData,
                Command::Output,
                Command::Loop {
                    body: vec![Command::IncrementPointer, Command::Output]
                }
            ],
            parse_valid(input)
        );
    }
}
