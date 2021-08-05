use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use thiserror::Error;

#[derive(Parser)]
#[grammar = "../grammar/brainfuck.pest"]
pub struct BrainfuckParser;

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
    let pairs = BrainfuckParser::parse(Rule::program, input)?;
    let program = pairs.peek().unwrap();
    println!("{:?}", pairs);
    let commands = parse_commands(program.into_inner());
    Ok(commands)
}

fn parse_commands(pairs: Pairs<Rule>) -> Vec<Command> {
    pairs.map(parse_command).filter_map(|c| c).collect()
}

fn parse_command(pair: Pair<Rule>) -> Option<Command> {
    match pair.as_rule() {
        Rule::pointer_increment => Some(Command::IncrementPointer),
        Rule::pointer_decrement => Some(Command::DecrementPointer),
        Rule::data_increment => Some(Command::IncrementData),
        Rule::data_decrement => Some(Command::DecrementData),
        Rule::input => Some(Command::Input),
        Rule::output => Some(Command::Output),
        Rule::loop_command => {
            let body = parse_commands(pair.into_inner());
            Some(Command::Loop { body })
        }
        Rule::simple_command | Rule::command => parse_command(pair.into_inner().next().unwrap()),
        Rule::comment => None,
        _ => panic!("Unexpected rule element {}", pair.as_str()),
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] pest::error::Error<Rule>),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_valid(input: &str) -> Vec<Command> {
        parse(input).expect("Invalid input")
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
