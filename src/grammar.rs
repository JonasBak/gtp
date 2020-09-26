use crate::parsing::*;
use regex::Regex;

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
                    ST::Repeated(Box::new(ST::Symbol(S::AST("DOC".into())))),
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
                        Box::new(ST::Symbol(L!("*".into()))),
                        Box::new(ST::Symbol(L!("?".into()))),
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

            >pluss    -> '+';
            >minus    -> '-';
            >multiply -> 'x';
            >divide   -> '/';
            >num      -> '\d+';
            "#
                .into()
            )
            .is_ok());
    }
}
