use crate::parsing::*;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rule in self.rules.iter() {
            write!(f, "{} -> {};", rule.name, rule.production)?;

            write!(f, "\n")?;
        }
        write!(f, "\n")?;
        for atom in self.atoms.iter() {
            match atom {
                Atom::Simple { name } => {
                    write!(f, ">{} -> '{}'", name, name)?;
                }
                Atom::Matched { name, m } => {
                    write!(f, ">{} -> '{:?}'", name, m)?;
                }
            }

            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolType::Symbol(i) => {
                write!(
                    f,
                    "{}",
                    match i {
                        Symbol::Lexem { t, .. } => t,
                        Symbol::AST(t) => t,
                    }
                )?;
            }
            SymbolType::Group(g) => {
                write!(f, "( ")?;
                for p in g.iter() {
                    write!(f, "{} ", p)?;
                }
                write!(f, ")")?;
            }
            SymbolType::Optional(o) => {
                write!(f, "{}?", o)?;
            }
            SymbolType::Repeated(m) => {
                write!(f, "{}*", m)?;
            }
            SymbolType::Switch(a, b) => {
                write!(f, "{} | {}", a, b)?;
            }
        }
        Ok(())
    }
}

pub fn get_parsing_grammar() -> Grammar {
    macro_rules! L {
        ( $t:expr ) => {
            Symbol::Lexem {
                t: $t,
                include_raw: false,
            }
        };
        ( $t:expr,  $i:expr ) => {
            Symbol::Lexem {
                t: $t,
                include_raw: $i,
            }
        };
    }
    type S = Symbol;
    type ST = SymbolType;
    Grammar {
        options: ParseOptions {
            ignore_newline: true,
            ignore_whitespace: true,
        },
        rules: vec![
            Rule {
                name: "START".into(),
                production: ST::Symbol(S::AST("DOC".into())),
            },
            Rule {
                name: "DOC".into(),
                production: ST::Group(vec![
                    ST::Switch(
                        Box::new(ST::Symbol(S::AST("EXP".into()))),
                        Box::new(ST::Symbol(S::AST("ATOM".into()))),
                    ),
                    ST::Symbol(L!(";".into())),
                    ST::Repeated(Box::new(ST::Group(vec![
                        ST::Switch(
                            Box::new(ST::Symbol(S::AST("EXP".into()))),
                            Box::new(ST::Symbol(S::AST("ATOM".into()))),
                        ),
                        ST::Symbol(L!(";".into())),
                    ]))),
                ]),
            },
            Rule {
                name: "ATOM".into(),
                production: ST::Group(vec![
                    ST::Symbol(L!(">".into())),
                    ST::Symbol(L!("ALPHA".into(), true)),
                    ST::Symbol(L!("->".into())),
                    ST::Symbol(L!("'".into())),
                    ST::Switch(
                        Box::new(ST::Symbol(L!("ALPHA".into(), true))),
                        Box::new(ST::Symbol(L!("LITERAL".into(), true))),
                    ),
                    ST::Symbol(L!("'".into())),
                ]),
            },
            Rule {
                name: "EXP".into(),
                production: ST::Group(vec![
                    ST::Symbol(L!("ALPHA".into(), true)),
                    ST::Symbol(L!("->".into())),
                    ST::Symbol(S::AST("PROD".into())),
                ]),
            },
            Rule {
                name: "PROD".into(),
                production: ST::Group(vec![
                    ST::Symbol(Symbol::AST("PROD_TERM".into())),
                    ST::Repeated(Box::new(ST::Group(vec![
                        ST::Optional(Box::new(ST::Symbol(L!("|".into(), true)))),
                        ST::Switch(
                            Box::new(ST::Symbol(S::AST("PROD_TERM".into()))),
                            Box::new(ST::Symbol(S::AST("PROD_GROUP".into()))),
                        ),
                    ]))),
                ]),
            },
            Rule {
                name: "PROD_TERM".into(),
                production: ST::Symbol(L!("ALPHA".into(), true)),
            },
            Rule {
                name: "PROD_GROUP".into(),
                production: ST::Group(vec![
                    ST::Symbol(L!("(".into())),
                    ST::Symbol(S::AST("PROD".into())),
                    ST::Symbol(L!(")".into())),
                    ST::Optional(Box::new(ST::Switch(
                        Box::new(ST::Symbol(L!("*".into(), true))),
                        Box::new(ST::Symbol(L!("?".into(), true))),
                    ))),
                ]),
            },
        ],
        atoms: vec![
            Atom::Simple { name: "|".into() },
            Atom::Simple { name: "(".into() },
            Atom::Simple { name: ")".into() },
            Atom::Simple { name: "*".into() },
            Atom::Simple { name: "?".into() },
            Atom::Simple { name: ";".into() },
            Atom::Simple { name: "->".into() },
            Atom::Simple { name: ">".into() },
            Atom::Simple { name: "'".into() },
            Atom::Matched {
                name: "NUMBER".into(),
                m: Regex::new(r"\d+").unwrap(),
            },
            Atom::Matched {
                name: "ALPHA".into(),
                m: Regex::new(r"\p{Alphabetic}+").unwrap(),
            },
            Atom::Matched {
                name: "LITERAL".into(),
                m: Regex::new(r"[^']+").unwrap(),
            },
        ],
    }
}

impl AST {
    fn assume_node(self) -> (String, Vec<AST>) {
        match self {
            AST::Node { t, children } => (t, children),
            _ => panic!(),
        }
    }
    fn assume_leaf(self) -> (String, String) {
        match self {
            AST::Leaf { t, raw } => (t, raw),
            _ => panic!(),
        }
    }
}

pub fn parse_ast_grammar(ast: AST) -> Grammar {
    let mut rules = Vec::new();
    let mut atoms = Vec::new();

    assert_eq!(ast.get_t(), "START");
    let (_, children) = ast.assume_node();
    assert_eq!(children.len(), 1);
    let ast = children.into_iter().nth(0).unwrap();
    assert_eq!(ast.get_t(), "DOC");

    let (_, children) = ast.assume_node();

    let mut doc = children.into_iter().peekable();

    loop {
        let (t, children) = doc.next().unwrap().assume_node();

        let mut c = children.into_iter();
        if t == "EXP" {
            let (_, name) = c.next().unwrap().assume_leaf();
            let production = parse_production(c.next().unwrap());
            rules.push(Rule { name, production });
        } else if t == "ATOM" {
            let (_, name) = c.next().unwrap().assume_leaf();
            let (_, literal) = c.next().unwrap().assume_leaf();
            atoms.push(Atom::Matched {
                name,
                m: Regex::new(&literal).unwrap(),
            });
        } else {
            panic!();
        }
        if doc.peek().is_none() {
            break;
        }
    }

    Grammar {
        options: ParseOptions::default(),
        rules,
        atoms,
    }
}

fn parse_production(ast: AST) -> SymbolType {
    match ast {
        AST::Node { t, children } => {
            let mut c = children.into_iter().peekable();
            if t == "PROD" {
                let mut children = vec![parse_production(c.next().unwrap())];
                while let Some(p) = c.next() {
                    if p.get_t() == "|" {
                        let mut rhs = Vec::new();
                        while c.peek().map(|p| p.get_t() != "|").unwrap_or(false) {
                            rhs.push(parse_production(c.next().unwrap()));
                        }
                        children = vec![SymbolType::Switch(
                            Box::new(SymbolType::Group(children)),
                            Box::new(SymbolType::Group(rhs)),
                        )];
                    } else {
                        children.push(parse_production(p));
                    }
                }
                SymbolType::Group(children)
            } else if t == "PROD_TERM" {
                parse_production(c.next().unwrap())
            } else if t == "PROD_GROUP" {
                let mut ast = parse_production(c.next().unwrap());
                if c.peek().is_some() {
                    let a = c.next().unwrap();
                    let t = a.get_t();
                    if t == "*" {
                        ast = SymbolType::Repeated(Box::new(ast));
                    } else if t == "?" {
                        ast = SymbolType::Optional(Box::new(ast));
                    }
                }
                ast
            } else {
                todo!("{}", t);
            }
        }
        AST::Leaf { t, raw } => {
            if t == "ALPHA" {
                if raw.to_ascii_uppercase() == raw {
                    SymbolType::Symbol(Symbol::AST(raw))
                } else {
                    SymbolType::Symbol(Symbol::Lexem {
                        t: raw,
                        include_raw: true,
                    })
                }
            } else {
                todo!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_simple_grammar() {
        let g = get_parsing_grammar();
        assert!(g
            .parse(
                &r#"
            START     -> PRODUCT;
            SUM       -> PRODUCT (OPA SUM)*;
            PRODUCT   -> NUMBER (OPB PRODUCT)*;
            NUMBER    -> num;
            NUMBER    -> minus num;

            OPA       -> pluss | minus;
            OPB       -> multiply | divide;

            >pluss    -> '\+';
            >minus    -> '-';
            >multiply -> 'x';
            >divide   -> '/';
            >num      -> '\d+';
            "#
                .into()
            )
            .is_ok());
    }
    #[test]
    fn parse_ast() {
        let g = get_parsing_grammar();
        let ast = g
            .parse(
                &r#"
            START     -> PRODUCT;
            SUM       -> PRODUCT (OPA SUM)*;
            PRODUCT   -> NUMBER (OPB PRODUCT)*;
            NUMBER    -> num;
            NUMBER    -> minus num;

            OPA       -> pluss | minus;
            OPB       -> multiply | divide;

            >pluss    -> '\+';
            >minus    -> '-';
            >multiply -> 'x';
            >divide   -> '/';
            >num      -> '\d+';
            "#
                .into(),
            )
            .unwrap();
        let gp = parse_ast_grammar(ast);

        assert!(gp.parse(&"1+2*3".into()).is_ok());
    }
}
