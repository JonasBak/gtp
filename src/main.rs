mod grammar;
mod parsing;

use clap::{App, Arg};
use grammar::*;
use parsing::*;
use std::fs;

fn get_line_from_pos(mut pos: usize, input: &String) -> (usize, usize, &str) {
    let mut lines = input.split("\n");
    let mut line_nr = 0;
    let mut prev_line = lines.next().unwrap();
    for line in lines {
        if line.len() > pos {
            break;
        } else {
            pos -= line.len() + 1;
            prev_line = line;
            line_nr += 1;
        }
    }
    return (pos, line_nr, prev_line);
}

fn print_error(err: ParseError, input: &String) {
    match err {
        ParseError::Lexem(pos, msg) | ParseError::Input(pos, msg) => {
            let (pos, line_nr, line) = get_line_from_pos(pos, input);
            eprintln!("{:>3}. | {}", line_nr + 1, line);
            eprintln!("     | {}^ {}", vec![" "; pos].join(""), msg);
        }
        ParseError::NoMatch(msg) => {
            eprintln!("{}", msg);
        }
    }
}

fn main() {
    env_logger::init();
    let g = get_parsing_grammar();
    let matches = App::new("generic text parser")
        .version("1.0")
        .arg(
            Arg::new("grammar")
                .about("file containing grammar")
                .required(true),
        )
        .arg(Arg::new("input").about("text to parse"))
        .get_matches();

    let raw_grammar = fs::read_to_string(matches.value_of("grammar").unwrap())
        .expect("could not read grammar file");
    let ast = g.parse(&raw_grammar).expect("could not parse grammar");
    let grammar = parse_ast_grammar(ast);
    match matches.value_of("input") {
        Some(input) => {
            let input = input.into();
            let ast = match grammar.parse(&input) {
                Ok(ast) => ast,
                Err(err) => {
                    print_error(err, &input);
                    std::process::exit(1);
                }
            };
            println!("{}", serde_json::to_string(&ast).unwrap());
        }
        None => {
            println!("Grammar parsed:\n{}", grammar);
        }
    }
}
