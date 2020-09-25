use regex::Regex;

#[derive(Debug)]
enum SymbolType {
    Symbol(Symbol),
    Group(Vec<SymbolType>),
    Optional(Box<SymbolType>),
    Multiple(Box<SymbolType>),
}

impl SymbolType {
    fn first_symbol(&self) -> &Symbol {
        match self {
            SymbolType::Symbol(i) => i,
            SymbolType::Group(g) => g[0].first_symbol(),
            SymbolType::Optional(o) => o.first_symbol(),
            SymbolType::Multiple(m) => m.first_symbol(),
        }
    }
}

#[derive(Debug)]
struct Rule {
    name: String,
    production: Vec<SymbolType>,
}

#[derive(Debug)]
struct Grammar {
    rules: Vec<Rule>,
    atoms: Vec<Atom>,
}

#[derive(Debug)]
enum Symbol {
    Lexem(String),
    AST(String),
}

#[derive(Debug)]
struct AST {
    t: String,
    children: Vec<AST>,
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
    fn first(&self, rule: &String) -> Vec<&String> {
        self.rules
            .iter()
            .filter(|r| r.name == *rule)
            .map(|r| match r.production[0].first_symbol() {
                Symbol::Lexem(f) => vec![f],
                Symbol::AST(r) => self.first(&r),
            })
            .flatten()
            .collect()
    }
    fn parse(&self, input: &String) -> Result<AST, ()> {
        log::debug!("parsing input:\n{}", input);

        let mut lexems = Lexem::iter(self, input);

        self.parse_rule(&"START".into(), &mut lexems)
    }
    fn parse_rule(&self, rule: &String, lexems: &mut LexemIter) -> Result<AST, ()> {
        let peeked = lexems.peek().ok_or(())?;
        log::debug!("parsing rule: {:?}", rule);
        log::debug!("peeked: {:?}", peeked);

        let rules = self
            .rules
            .iter()
            .filter(|Rule { name, .. }| name == rule)
            .collect::<Vec<_>>();

        if let Some(Rule { production, .. }) = rules
            .iter()
            .find(|r| self.first(&r.name).contains(&&peeked.t))
        {
            let mut children = Vec::new();
            for step in production.iter() {
                children.extend(self.parse_symbol_type(step, lexems)?);
            }
            return Ok(AST {
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
            SymbolType::Optional(o) => todo!(),
            SymbolType::Multiple(m) => todo!(),
        }
        Ok(parsed)
    }
    fn parse_symbol(&self, s: &Symbol, lexems: &mut LexemIter) -> Result<Option<AST>, ()> {
        match s {
            Symbol::Lexem(la) => {
                if lexems.peek().map(|p| p.t == *la).unwrap_or(false) {
                    lexems.next();
                    Ok(None)
                } else {
                    Err(())
                }
            }
            Symbol::AST(rule) => Ok(Some(self.parse_rule(rule, lexems)?)),
        }
    }
}

#[derive(Debug)]
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
        }
    }
}

struct LexemIter<'a> {
    grammar: &'a Grammar,
    input: &'a String,
    cursor: usize,
    ok: Result<(), ()>,
    peeked: Option<Lexem>,
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
        match self.grammar.match_input(&self.input[self.cursor..]) {
            Some((lexem, i)) => {
                self.cursor += i;
                Some(lexem)
            }
            None => {
                self.ok = Err(());
                None
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
enum Atom {
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

fn main() {
    env_logger::init();

    let g = Grammar {
        rules: vec![
            Rule {
                name: "START".into(),
                production: vec![SymbolType::Symbol(Symbol::AST("PAR".into()))],
            },
            Rule {
                name: "PAR".into(),
                production: vec![
                    SymbolType::Symbol(Symbol::Lexem("(".into())),
                    SymbolType::Symbol(Symbol::AST("YEET".into())),
                    SymbolType::Symbol(Symbol::Lexem(")".into())),
                ],
            },
            Rule {
                name: "YEET".into(),
                production: vec![SymbolType::Symbol(Symbol::Lexem("yeet".into()))],
            },
        ],
        atoms: vec![
            Atom::Simple { name: "(".into() },
            Atom::Simple { name: ")".into() },
            Atom::Simple {
                name: "yeet".into(),
            },
        ],
    };
    println!("{:?}", g.parse(&"(yeet)".into()));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_lexem_iter() {
        let g = Grammar {
            rules: vec![],
            atoms: vec![
                Atom::Simple { name: "(".into() },
                Atom::Simple { name: ")".into() },
            ],
        };
        let input = "(()())".into();
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
        let input = "(1234)".into();
        let mut lexem_iter = Lexem::iter(&g, &input);
        assert_eq!(lexem_iter.next().unwrap().t, "(");

        let n = lexem_iter.next().unwrap();
        assert_eq!(n.t, "NUMBER");
        assert_eq!(n.raw, "1234");

        assert_eq!(lexem_iter.next().unwrap().t, ")");
        assert!(lexem_iter.next().is_none());
    }
    #[test]
    fn simple_parse() {
        let g = Grammar {
            rules: vec![
                Rule {
                    name: "START".into(),
                    production: vec![SymbolType::Symbol(Symbol::AST("PAR".into()))],
                },
                Rule {
                    name: "PAR".into(),
                    production: vec![
                        SymbolType::Symbol(Symbol::Lexem("(".into())),
                        SymbolType::Symbol(Symbol::Lexem("NUMBER".into())),
                        SymbolType::Symbol(Symbol::Lexem(")".into())),
                    ],
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
        assert!(g.parse(&"".into()).is_err());
        assert!(g.parse(&"(()".into()).is_err());
        assert!(g.parse(&"()".into()).is_err());
        assert!(g.parse(&"1424)".into()).is_err());
        assert!(g.parse(&"(1424".into()).is_err());
    }
}
