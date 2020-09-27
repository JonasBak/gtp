use regex::Regex;

#[derive(Debug)]
pub enum SymbolType {
    Symbol(Symbol),
    Group(Vec<SymbolType>),
    Optional(Box<SymbolType>),
    Repeated(Box<SymbolType>),
    Switch(Box<SymbolType>, Box<SymbolType>),
}

impl SymbolType {
    fn nullable(&self) -> bool {
        match self {
            SymbolType::Symbol(_) | SymbolType::Group(_) => false,
            SymbolType::Switch(a, b) => a.nullable() || b.nullable(),
            SymbolType::Optional(_) | SymbolType::Repeated(_) => true,
        }
    }
}

impl SymbolType {
    fn first_symbol(&self) -> Vec<&Symbol> {
        match self {
            SymbolType::Symbol(i) => vec![i],
            SymbolType::Group(g) => {
                let mut f = Vec::new();
                for s in g.iter() {
                    let first = s.first_symbol();
                    f.extend(first);
                    if !s.nullable() {
                        break;
                    }
                }
                f
            }
            SymbolType::Optional(o) => o.first_symbol(),
            SymbolType::Repeated(m) => m.first_symbol(),
            SymbolType::Switch(a, b) => {
                let mut v = a.first_symbol();
                v.extend(b.first_symbol());
                v
            }
        }
    }
}

#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub production: SymbolType,
}

#[derive(Debug, Copy, Clone)]
pub struct ParseOptions {
    pub ignore_whitespace: bool,
    pub ignore_newline: bool,
}

impl ParseOptions {
    pub fn default() -> Self {
        ParseOptions {
            ignore_whitespace: false,
            ignore_newline: false,
        }
    }
}

#[derive(Debug)]
pub struct Grammar {
    pub rules: Vec<Rule>,
    pub atoms: Vec<Atom>,

    pub options: ParseOptions,
}

#[derive(Debug)]
pub enum Symbol {
    Lexem { t: String, include_raw: bool },
    AST(String),
}

#[derive(Debug)]
pub enum AST {
    Node { t: String, children: Vec<AST> },
    Leaf { t: String, raw: String },
}

impl AST {
    pub fn get_t(&self) -> &String {
        match self {
            AST::Node { t, .. } => t,
            AST::Leaf { t, .. } => t,
        }
    }
}

impl Grammar {
    fn match_input(&self, input: &str) -> Option<(Lexem, usize)> {
        self.atoms
            .iter()
            .find_map(|atom| atom.match_input(input))
            .map(|(name, i)| {
                (
                    Lexem {
                        t: name,
                        raw: String::from(&input[0..i]),
                    },
                    i,
                )
            })
    }
    fn first_from_rule(&self, rule: &String) -> Vec<&String> {
        self.rules
            .iter()
            .filter(|r| r.name == *rule)
            .map(|r| {
                r.production
                    .first_symbol()
                    .iter()
                    .map(|s| match s {
                        Symbol::Lexem { t, .. } => vec![t],
                        Symbol::AST(r) => self.first_from_rule(&r),
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
    fn first_from_symbol<'a>(&'a self, s: &'a Symbol) -> Vec<&'a String> {
        match s {
            Symbol::Lexem { t, .. } => vec![t],
            Symbol::AST(r) => self.first_from_rule(r),
        }
    }
    fn production_matches_lexem(&self, p: &SymbolType, t: &String) -> bool {
        p.first_symbol()
            .iter()
            .map(|s| self.first_from_symbol(s).contains(&t))
            .fold(false, |a, b| a || b)
    }
    pub fn parse(&self, input: &String) -> Result<AST, ()> {
        log::debug!("parsing input:\n{}", input);

        let mut lexems = Lexem::iter(self, input);

        let ast = self.parse_rule(&"START".into(), &mut lexems)?;

        if lexems.next().is_some() {
            return Err(());
        }
        Ok(ast)
    }
    fn parse_rule(&self, rule: &String, lexems: &mut LexemIter) -> Result<AST, ()> {
        let peeked = lexems.peek().ok_or(()).expect("todo handle empty");
        log::debug!("parsing rule: {:?}", rule);
        log::debug!("peeked: {:?}", peeked);

        let rules = self
            .rules
            .iter()
            .filter(|Rule { name, .. }| name == rule)
            .collect::<Vec<_>>();

        if rules.len() == 0 {
            panic!("no rule matching name: {}", rule);
        }

        log::debug!("rules found: {:?}", rules);

        if let Some(Rule { production, .. }) = rules
            .iter()
            .find(|r| self.production_matches_lexem(&r.production, &peeked.t))
        {
            log::debug!("choosing production: {:?}", production);

            let children = self.parse_symbol_type(production, lexems)?;
            return Ok(AST::Node {
                t: rule.clone(),
                children,
            });
        }

        return Err(());
    }
    fn parse_symbol_type(&self, s: &SymbolType, lexems: &mut LexemIter) -> Result<Vec<AST>, ()> {
        let mut parsed = Vec::new();
        match s {
            SymbolType::Symbol(s) => {
                if let Some(ast) = self.parse_symbol(s, lexems)? {
                    parsed.push(ast);
                }
            }
            SymbolType::Group(g) => {
                for s in g.iter() {
                    parsed.extend(self.parse_symbol_type(s, lexems)?);
                }
            }
            SymbolType::Optional(o) => {
                if let Some(p) = lexems.peek() {
                    if self.production_matches_lexem(o, &p.t) {
                        parsed.extend(self.parse_symbol_type(o, lexems)?);
                    }
                }
            }
            SymbolType::Repeated(m) => {
                while let Some(p) = lexems.peek() {
                    if self.production_matches_lexem(m, &p.t) {
                        parsed.extend(self.parse_symbol_type(m, lexems)?);
                    } else {
                        break;
                    }
                }
            }
            SymbolType::Switch(a, b) => {
                if let Some(p) = lexems.peek() {
                    if self.production_matches_lexem(a, &p.t) {
                        parsed.extend(self.parse_symbol_type(a, lexems)?);
                    } else {
                        parsed.extend(self.parse_symbol_type(b, lexems)?);
                    }
                } else {
                    return Err(());
                }
            }
        }
        Ok(parsed)
    }
    fn parse_symbol(&self, s: &Symbol, lexems: &mut LexemIter) -> Result<Option<AST>, ()> {
        match s {
            Symbol::Lexem { t, include_raw } => {
                if lexems.peek().map(|p| p.t == *t).unwrap_or(false) {
                    let a = lexems.next().unwrap();
                    if *include_raw {
                        Ok(Some(AST::Leaf { t: a.t, raw: a.raw }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Err(())
                }
            }
            Symbol::AST(rule) => Ok(Some(self.parse_rule(rule, lexems)?)),
        }
    }
}

#[derive(Debug, Clone)]
struct Lexem {
    t: String,
    raw: String,
}

impl Lexem {
    fn iter<'a>(grammar: &'a Grammar, input: &'a String) -> LexemIter<'a> {
        LexemIter {
            grammar,
            input,
            cursor: 0,
            ok: Ok(()),
            peeked: None,
            options: grammar.options,
        }
    }
}

#[derive(Clone)]
struct LexemIter<'a> {
    grammar: &'a Grammar,
    input: &'a String,
    cursor: usize,
    ok: Result<(), ()>,
    peeked: Option<Lexem>,
    options: ParseOptions,
}

impl LexemIter<'_> {
    fn peek(&mut self) -> Option<&Lexem> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }
        self.peeked = self.shift();
        self.peeked.as_ref()
    }
    fn shift(&mut self) -> Option<Lexem> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }
        if self.cursor >= self.input.len() {
            return None;
        }
        self.skip_ignored();
        match self.grammar.match_input(&self.input[self.cursor..]) {
            Some((lexem, i)) => {
                self.cursor += i;
                self.skip_ignored();
                Some(lexem)
            }
            None => {
                self.ok = Err(());
                None
            }
        }
    }
    fn skip_ignored(&mut self) {
        while self.cursor < self.input.len() {
            let c = self.input.chars().nth(self.cursor).unwrap();
            if c == ' ' && self.options.ignore_whitespace
                || c == '\n' && self.options.ignore_newline
            {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }
}

impl Iterator for LexemIter<'_> {
    type Item = Lexem;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.shift();
        log::debug!("next lexem: {:?}", n);
        n
    }
}

#[derive(Debug)]
pub enum Atom {
    Simple { name: String },
    Matched { name: String, m: Regex },
}

impl Atom {
    fn match_input(&self, input: &str) -> Option<(String, usize)> {
        match self {
            Atom::Simple { name } => {
                if input.starts_with(name) {
                    return Some((name.clone(), name.len()));
                }
            }
            Atom::Matched { name, m } => {
                let m = m.find(input)?;
                if m.start() != 0 {
                    return None;
                }
                return Some((name.clone(), m.end()));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_lexem_iter() {
        let g = Grammar {
            options: ParseOptions {
                ignore_whitespace: true,
                ignore_newline: false,
            },
            rules: vec![],
            atoms: vec![
                Atom::Simple { name: "(".into() },
                Atom::Simple { name: ")".into() },
            ],
        };
        let input = "(() ())".into();
        let mut lexem_iter = Lexem::iter(&g, &input);
        assert_eq!(lexem_iter.next().unwrap().t, "(");
        assert_eq!(lexem_iter.next().unwrap().t, "(");
        assert_eq!(lexem_iter.next().unwrap().t, ")");
        assert_eq!(lexem_iter.next().unwrap().t, "(");
        assert_eq!(lexem_iter.next().unwrap().t, ")");
        assert_eq!(lexem_iter.next().unwrap().t, ")");
        assert!(lexem_iter.next().is_none());
    }
    #[test]
    fn combined_lexem_iter() {
        let g = Grammar {
            options: ParseOptions {
                ignore_whitespace: true,
                ignore_newline: true,
            },
            rules: vec![],
            atoms: vec![
                Atom::Simple { name: "(".into() },
                Atom::Simple { name: ")".into() },
                Atom::Matched {
                    name: "NUMBER".into(),
                    m: Regex::new(r"\d+").unwrap(),
                },
            ],
        };
        let input = "(\n1234 )".into();
        let mut lexem_iter = Lexem::iter(&g, &input);
        assert_eq!(lexem_iter.next().unwrap().t, "(");

        let n = lexem_iter.next().unwrap();
        assert_eq!(n.t, "NUMBER");
        assert_eq!(n.raw, "1234");

        assert_eq!(lexem_iter.next().unwrap().t, ")");
        assert!(lexem_iter.next().is_none());
    }
    #[test]
    fn parse_simple() {
        let g = Grammar {
            options: ParseOptions::default(),
            rules: vec![
                Rule {
                    name: "START".into(),
                    production: SymbolType::Symbol(Symbol::AST("PAR".into())),
                },
                Rule {
                    name: "PAR".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "(".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "NUMBER".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: ")".into(),
                            include_raw: false,
                        }),
                    ]),
                },
            ],
            atoms: vec![
                Atom::Simple { name: "(".into() },
                Atom::Simple { name: ")".into() },
                Atom::Matched {
                    name: "NUMBER".into(),
                    m: Regex::new(r"\d+").unwrap(),
                },
            ],
        };
        assert!(g.parse(&"(1424)".into()).is_ok());
        assert!(g.parse(&"(()".into()).is_err());
        assert!(g.parse(&"()".into()).is_err());
        assert!(g.parse(&"1424)".into()).is_err());
        assert!(g.parse(&"(1424".into()).is_err());
    }
    #[test]
    fn parse_optional() {
        let g = Grammar {
            options: ParseOptions::default(),
            rules: vec![
                Rule {
                    name: "START".into(),
                    production: SymbolType::Symbol(Symbol::AST("FLOAT".into())),
                },
                Rule {
                    name: "FLOAT".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "NUMBER".into(),
                            include_raw: false,
                        }),
                        SymbolType::Optional(Box::new(SymbolType::Group(vec![
                            SymbolType::Symbol(Symbol::Lexem {
                                t: ".".into(),
                                include_raw: false,
                            }),
                            SymbolType::Symbol(Symbol::Lexem {
                                t: "NUMBER".into(),
                                include_raw: false,
                            }),
                        ]))),
                    ]),
                },
            ],
            atoms: vec![
                Atom::Simple { name: ".".into() },
                Atom::Matched {
                    name: "NUMBER".into(),
                    m: Regex::new(r"\d+").unwrap(),
                },
            ],
        };
        assert!(g.parse(&"12.34".into()).is_ok());
        assert!(g.parse(&"12".into()).is_ok());
        assert!(g.parse(&"12.".into()).is_err());
    }
    #[test]
    fn parse_multiple() {
        let g = Grammar {
            options: ParseOptions::default(),
            rules: vec![
                Rule {
                    name: "START".into(),
                    production: SymbolType::Symbol(Symbol::AST("PARS".into())),
                },
                Rule {
                    name: "PARS".into(),
                    production: SymbolType::Repeated(Box::new(SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "(".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: ")".into(),
                            include_raw: false,
                        }),
                    ]))),
                },
            ],
            atoms: vec![
                Atom::Simple { name: "(".into() },
                Atom::Simple { name: ")".into() },
            ],
        };
        assert!(g.parse(&"()".into()).is_ok());
        assert!(g.parse(&"()()".into()).is_ok());
        assert!(g.parse(&"()()()".into()).is_ok());
        assert!(g.parse(&"()(".into()).is_err());
        assert!(g.parse(&"()()(".into()).is_err());
    }
    #[test]
    fn parse_multiple_matching_rules() {
        let g = Grammar {
            options: ParseOptions::default(),
            rules: vec![
                Rule {
                    name: "START".into(),
                    production: SymbolType::Symbol(Symbol::AST("LIST".into())),
                },
                Rule {
                    name: "START".into(),
                    production: SymbolType::Symbol(Symbol::AST("OBJ".into())),
                },
                Rule {
                    name: "LIST".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "[".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "]".into(),
                            include_raw: false,
                        }),
                    ]),
                },
                Rule {
                    name: "OBJ".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "{".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "}".into(),
                            include_raw: false,
                        }),
                    ]),
                },
            ],
            atoms: vec![
                Atom::Simple { name: "[".into() },
                Atom::Simple { name: "]".into() },
                Atom::Simple { name: "{".into() },
                Atom::Simple { name: "}".into() },
                Atom::Matched {
                    name: "NUMBER".into(),
                    m: Regex::new(r"\d+").unwrap(),
                },
            ],
        };
        assert!(g.parse(&"[]".into()).is_ok());
        assert!(g.parse(&"{}".into()).is_ok());
        assert!(g.parse(&"[}".into()).is_err());
    }
    #[test]
    fn parse_switch() {
        let g = Grammar {
            options: ParseOptions::default(),
            rules: vec![
                Rule {
                    name: "START".into(),
                    production: SymbolType::Symbol(Symbol::AST("COMP".into())),
                },
                Rule {
                    name: "COMP".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "NUMBER".into(),
                            include_raw: false,
                        }),
                        SymbolType::Switch(
                            Box::new(SymbolType::Symbol(Symbol::Lexem {
                                t: "<".into(),
                                include_raw: false,
                            })),
                            Box::new(SymbolType::Symbol(Symbol::Lexem {
                                t: ">".into(),
                                include_raw: false,
                            })),
                        ),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "NUMBER".into(),
                            include_raw: false,
                        }),
                    ]),
                },
            ],
            atoms: vec![
                Atom::Simple { name: "<".into() },
                Atom::Simple { name: ">".into() },
                Atom::Matched {
                    name: "NUMBER".into(),
                    m: Regex::new(r"\d+").unwrap(),
                },
            ],
        };
        assert!(g.parse(&"12<9".into()).is_ok());
        assert!(g.parse(&"12>9".into()).is_ok());
        assert!(g.parse(&"12".into()).is_err());
    }
    #[test]
    fn parse_mini_json() {
        let g = Grammar {
            options: ParseOptions::default(),
            rules: vec![
                Rule {
                    name: "START".into(),
                    production: SymbolType::Symbol(Symbol::AST("ITEM".into())),
                },
                Rule {
                    name: "ITEM".into(),
                    production: SymbolType::Symbol(Symbol::AST("OBJ".into())),
                },
                Rule {
                    name: "ITEM".into(),
                    production: SymbolType::Symbol(Symbol::AST("LIST".into())),
                },
                Rule {
                    name: "ITEM".into(),
                    production: SymbolType::Symbol(Symbol::Lexem {
                        t: "NUMBER".into(),
                        include_raw: false,
                    }),
                },
                Rule {
                    name: "OBJ".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "{".into(),
                            include_raw: false,
                        }),
                        SymbolType::Optional(Box::new(SymbolType::Group(vec![
                            SymbolType::Symbol(Symbol::AST("KV".into())),
                            SymbolType::Repeated(Box::new(SymbolType::Group(vec![
                                SymbolType::Symbol(Symbol::Lexem {
                                    t: ",".into(),
                                    include_raw: false,
                                }),
                                SymbolType::Symbol(Symbol::AST("KV".into())),
                            ]))),
                        ]))),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "}".into(),
                            include_raw: false,
                        }),
                    ]),
                },
                Rule {
                    name: "KV".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "\"".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "STRING".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "\"".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: ":".into(),
                            include_raw: false,
                        }),
                        SymbolType::Symbol(Symbol::AST("ITEM".into())),
                    ]),
                },
                Rule {
                    name: "LIST".into(),
                    production: SymbolType::Group(vec![
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "[".into(),
                            include_raw: false,
                        }),
                        SymbolType::Optional(Box::new(SymbolType::Group(vec![
                            SymbolType::Symbol(Symbol::AST("ITEM".into())),
                            SymbolType::Repeated(Box::new(SymbolType::Group(vec![
                                SymbolType::Symbol(Symbol::Lexem {
                                    t: ",".into(),
                                    include_raw: false,
                                }),
                                SymbolType::Symbol(Symbol::AST("ITEM".into())),
                            ]))),
                        ]))),
                        SymbolType::Symbol(Symbol::Lexem {
                            t: "]".into(),
                            include_raw: false,
                        }),
                    ]),
                },
            ],
            atoms: vec![
                Atom::Simple { name: "{".into() },
                Atom::Simple { name: "}".into() },
                Atom::Simple { name: "[".into() },
                Atom::Simple { name: "]".into() },
                Atom::Simple { name: ",".into() },
                Atom::Simple { name: ":".into() },
                Atom::Simple { name: "\"".into() },
                Atom::Matched {
                    name: "STRING".into(),
                    m: Regex::new(r"\p{Alphabetic}+").unwrap(),
                },
                Atom::Matched {
                    name: "NUMBER".into(),
                    m: Regex::new(r"\d+").unwrap(),
                },
            ],
        };
        assert!(g.parse(&"{}".into()).is_ok());
        assert!(g.parse(&"[]".into()).is_ok());
        assert!(g.parse(&r#"{"field":12}"#.into()).is_ok());
        assert!(g.parse(&r#"{"fieldA":[1,2,3],"fieldB":{}}"#.into()).is_ok());
        assert!(g.parse(&"[{},12,[[]]]".into()).is_ok());
        assert!(g.parse(&"[".into()).is_err());
        assert!(g.parse(&"[{{}}]".into()).is_err());
        assert!(g.parse(&r#"{"field"}"#.into()).is_err());
    }
}
