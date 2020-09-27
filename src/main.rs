mod grammar;
mod parsing;

use clap::{App, Arg};
use grammar::*;
use std::fs;

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
            println!(
                "{}",
                serde_json::to_string(
                    &grammar.parse(&input.into()).expect("Could not parse input")
                )
                .unwrap()
            );
        }
        None => {
            println!("Grammar parsed:\n{}", grammar);
        }
    }
}
