//! You can use this file for experiments.
//! Run this file with `cargo test --test 05_brainfuck_interpreter`.

// TODO (bonus): Create an interpreter for the [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) language.
// The Brainfuck program will be parsed out of a string and represented as a struct.
//
// Handle both parsing and execution errors using enums representing error conditions,
// see tests for details.
// A parsing error can be either an unknown instruction or an unpaired loop instruction.
// An execution error can be either that the program tries to read input, but there is no more
// input available, or when the program executes more than 10000 instructions (which probably
// signals an infinite loop).
//
// Hint: Put `#[derive(Debug, Eq, PartialEq)]` on top of `ParseError`, `ExecuteError` and `Program`
// (and any other custom types nested inside them) so that asserts in tests work.

use std::str;

#[derive(Debug, Eq, PartialEq)]
enum ParseError {
    UnknownInstruction { location: usize, instruction: char },
    UnmatchedLoop { location: usize },
}

#[derive(Debug, Eq, PartialEq)]
enum ExecuteError {
    NoInputLeft,
    InfiniteLoop,
}

#[derive(Debug, Eq, PartialEq)]
struct Program {
    instructions: Vec<Instruction>,
}

#[derive(Debug, Eq, PartialEq)]
enum Instruction {
    Left,
    Right,
    Add,
    Sub,
    Print,
    Save,
    LoopBegin,
    LoopEnd(usize),
    Noop,
    End,
}

impl Program {
    fn execute(&self, input: Vec<u8>, mut tape: Vec<u8>) -> Result<String, ExecuteError> {
        let mut output = String::new();
        let mut pointer: usize = 0;
        let mut ip = 0;
        let mut ic = 0;
        let mut input_pointer: usize = 0;

        if !self.instructions.is_empty() {
            loop {
                match self.instructions[ip] {
                    Instruction::Left => {
                        pointer -= 1;
                    }
                    Instruction::Right => {
                        pointer += 1;
                    }
                    Instruction::Add => {
                        tape[pointer] += 1;
                    }
                    Instruction::Sub => {
                        tape[pointer] -= 1;
                    }
                    Instruction::Print => {
                        output.push(tape[pointer] as char);
                    }
                    Instruction::Save => {
                        if input_pointer >= input.len() {
                            return Err(ExecuteError::NoInputLeft);
                        }
                        tape[pointer] = input[input_pointer];
                        input_pointer += 1;
                    }
                    Instruction::LoopEnd(new_ip) => {
                        if tape[pointer] != 0 {
                            if ic >= 10000 {
                                return Err(ExecuteError::InfiniteLoop);
                            }
                            ip = new_ip - 1;
                        }
                    }
                    Instruction::LoopBegin | Instruction::Noop => {}
                    Instruction::End => {
                        return Ok(output);
                    }
                }
                ip += 1;
                ic += 1;
            }
        }
        Ok(output)
    }
}

fn parse_program(str: &str) -> Result<Program, ParseError> {
    let mut program = Program {
        instructions: Vec::new(),
    };
    let mut loop_starts: Vec<usize> = Vec::new();

    for (i, char) in str.chars().enumerate() {
        program.instructions.push(match char {
            '>' => Instruction::Right,
            '<' => Instruction::Left,
            '+' => Instruction::Add,
            '-' => Instruction::Sub,
            '.' => Instruction::Print,
            ',' => Instruction::Save,
            ' ' | '\n' | '\r' => Instruction::Noop,
            '[' => {
                loop_starts.push(i);
                Instruction::LoopBegin
            }
            ']' => {
                let last_start = loop_starts.pop();
                match last_start {
                    Some(index) => Instruction::LoopEnd(index),
                    None => return Err(ParseError::UnmatchedLoop { location: i }),
                }
            }
            _ => {
                return Err(ParseError::UnknownInstruction {
                    location: i,
                    instruction: char,
                })
            }
        });
    }
    program.instructions.push(Instruction::End);
    match loop_starts.pop() {
        Some(i) => Err(ParseError::UnmatchedLoop { location: i }),
        None => Ok(program),
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::{parse_program, ExecuteError, ParseError};

    #[test]
    fn parse_empty() {
        check_output("", "", "");
    }

    #[test]
    fn parse_unknown_instruction() {
        assert!(matches!(
            parse_program(">p"),
            Err(ParseError::UnknownInstruction {
                location: 1,
                instruction: 'p'
            })
        ));
    }

    #[test]
    fn parse_unmatched_loop_start() {
        assert_eq!(
            parse_program(">++[+>][++>"),
            Err(ParseError::UnmatchedLoop { location: 7 })
        );
    }

    #[test]
    fn parse_unmatched_loop_end() {
        assert_eq!(
            parse_program(">++[+>][++>]+]"),
            Err(ParseError::UnmatchedLoop { location: 13 })
        );
    }

    #[test]
    fn missing_input() {
        let program = parse_program(",").unwrap();
        let result = program.execute(vec![], vec![0; 30000]);
        assert_eq!(result, Err(ExecuteError::NoInputLeft));
    }

    #[test]
    fn infinite_loop() {
        let program = parse_program("+[]").unwrap();
        let result = program.execute(vec![], vec![0; 30000]);
        assert_eq!(result, Err(ExecuteError::InfiniteLoop));
    }

    #[test]
    fn copy_input() {
        check_output(",.>,.>,.>,.>,.", "hello", "hello");
    }

    #[test]
    fn output_exclamation_mark() {
        check_output("+++++++++++++++++++++++++++++++++.", "", "!");
    }

    #[test]
    fn three_exclamation_marks() {
        check_output(">+++++++++++++++++++++++++++++++++<+++[>.<-]", "", "!!!");
    }

    #[test]
    fn hello_world() {
        check_output("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.", "", "Hello World!\n");
    }

    fn check_output(program_text: &str, input: &str, expected_output: &str) {
        let program = parse_program(program_text);
        match program {
            Ok(program) => {
                let result = program
                    .execute(input.to_string().into_bytes(), vec![0; 30000])
                    .expect(&format!("Cannot execute program {program_text}"));
                assert_eq!(result, expected_output);
            }
            Err(error) => {
                panic!("Error occurred while parsing program {program_text}: {error:?}");
            }
        }
    }
}
