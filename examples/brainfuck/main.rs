use gtp::*;
use std::env;

const GRAMMAR: &str = r#"
START  -> (OP (START)?)
OP     -> (lb START rb)
OP     -> (pluss)
OP     -> (minus)
OP     -> (dot)
OP     -> (comma)
OP     -> (left)
OP     -> (right)

>lb    -> '\['
>rb    -> '\]'
>pluss -> '\+'
>minus -> '-'
>dot   -> '\.'
>comma -> ','
>left  -> '[<]'
>right -> '[>]'
"#;

fn main() {
    let mut args = env::args();
    args.next();
    let input = match args.next() {
        Some(file) => {
            std::fs::read_to_string(file).expect("could not read file")
        }
        None => {
            "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".into()
        }
    };

    let g = get_parsing_grammar();
    let ast = g.parse(&GRAMMAR.into()).unwrap();
    let gp = parse_ast_grammar(ast).with_options(ParseOptions {
        ignore_newline: true,
        ignore_whitespace: true,
        bubble_intermediate: true,
    });
    let ast = gp.parse(&input).unwrap();
    Interpreter::run(256, &ast);
}

struct Interpreter {
    tape: Vec<u8>,
    ptr: usize,
}

impl Interpreter {
    fn run(width: usize, ast: &AST) {
        Interpreter {
            tape: vec![0; width],
            ptr: 0,
        }
        .interpret(&ast);
    }
    fn interpret(&mut self, ast: &AST) {
        match ast {
            AST::Node { t, children } => {
                let mut children = children.iter();
                match t.as_str() {
                    "START" => {
                        while let Some(next) = children.next() {
                            self.interpret(next);
                        }
                    }
                    "OP" => {
                        // assert because bubble_intermediate = true
                        assert_eq!(children.next().unwrap().get_t(), "lb");
                        let body = children.next().unwrap();
                        assert_eq!(children.next().unwrap().get_t(), "rb");
                        while self.tape[self.ptr] != 0 {
                            self.interpret(body);
                        }
                    }
                    _ => panic!(),
                }
            }
            AST::Leaf { t, .. } => match t.as_str() {
                "pluss" => self.tape[self.ptr] += 1,
                "minus" => self.tape[self.ptr] -= 1,
                "dot" => print!("{}", self.tape[self.ptr] as char),
                "comma" => todo!(),
                "left" => self.ptr -= 1,
                "right" => self.ptr += 1,
                _ => panic!(),
            },
        }
    }
}
