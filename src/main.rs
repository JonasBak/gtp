mod grammar;
mod parsing;

use clap::Clap;
use grammar::*;
use parsing::*;
use std::fs;
use std::io::{self, Read};

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

fn print_output(ast: &AST, format: &Format) {
    match format {
        Format::Json => {
            println!("{}", serde_json::to_string(&ast).unwrap());
        }
        Format::Yaml => {
            println!("{}", serde_yaml::to_string(&ast).unwrap());
        }
    }
}

/// Parse input text with provided grammar, output parsed syntax tree
#[derive(Clap)]
struct Opts {
    /// File containing grammar
    grammar: String,

    /// Format of output
    #[clap(short, long, default_value = "json")]
    output: Format,

    // input types
    /// Input to parse
    input: Option<String>,
    /// Read input text from file instead of arg
    #[clap(short, long)]
    input_file: Option<String>,
    /// Read input text from stdin
    #[clap(long)]
    stdin: bool,

    // parse options:
    /// Set all ignore options to true
    #[clap(long)]
    ignore_all: bool,
    /// Skip newlines in input
    #[clap(long)]
    ignore_newline: bool,
    /// Skip whitespaces in input
    #[clap(long)]
    ignore_whitespace: bool,
    /// Remove intermediate nodes in the ast with only one child, making the child "bubble up"
    #[clap(long)]
    bubble: bool,
}

enum Format {
    Json,
    Yaml,
}

impl std::str::FromStr for Format {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Format::Json),
            "yml" | "yaml" => Ok(Format::Yaml),
            _ => Err("no match"),
        }
    }
}

fn main() {
    env_logger::init();

    let opts: Opts = Opts::parse();

    let g = get_parsing_grammar();

    let raw_grammar = fs::read_to_string(opts.grammar).expect("could not read grammar file");

    let ast = match g.parse(&raw_grammar) {
        Ok(ast) => ast,
        Err(err) => {
            print_error(err, &raw_grammar);
            std::process::exit(1);
        }
    };

    let options = {
        let mut o = ParseOptions::default();
        let all = opts.ignore_all;
        o.ignore_newline = opts.ignore_newline || all;
        o.ignore_whitespace = opts.ignore_whitespace || all;
        o.bubble_intermediate = opts.bubble;
        o
    };

    let grammar = parse_ast_grammar(ast).with_options(options);

    let input = if let Some(input) = opts.input {
        Some(input)
    } else if let Some(input_file) = opts.input_file {
        Some(fs::read_to_string(input_file).expect("could not read input file"))
    } else if opts.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("could not read input file");
        Some(buffer)
    } else {
        None
    };

    if let Some(input) = input {
        let ast = match grammar.parse(&input) {
            Ok(ast) => ast,
            Err(err) => {
                print_error(err, &input);
                std::process::exit(1);
            }
        };
        print_output(&ast, &opts.output);
    } else {
        println!("Grammar parsed:\n{}", grammar);
    }
}
