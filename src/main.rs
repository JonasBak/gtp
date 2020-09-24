use regex::Regex;

#[derive(Debug)]
struct Rule {
    name: String,
    production: Vec<IN>,
}

#[derive(Debug)]
struct Grammar {
    rules: Vec<Rule>,
    atoms: Vec<Atom>,
}

#[derive(Debug)]
enum IN {
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
    fn first(&self, rule: &String) -> &String {
        match &self
            .rules
            .iter()
            .find(|r| r.name == *rule)
            .unwrap()
            .production[0]
        {
            IN::Lexem(f) => &f,
            IN::AST(r) => self.first(&r),
        }
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

        if let Some(Rule { production, .. }) =
            rules.iter().find(|r| self.first(&r.name) == &peeked.t)
        {
            let mut children = Vec::new();
            for step in production.iter() {
                match step {
                    IN::Lexem(la) => {
                        if lexems.peek().map(|p| p.t == *la).unwrap_or(false) {
                            lexems.next();
                        } else {
                            log::debug!("err {:?} {:?}", rule, lexems.peek());
                            return Err(());
                        }
                    }
                    IN::AST(rule) => {
                        children.push(self.parse_rule(rule, lexems)?);
                    }
                }
            }
            return Ok(AST {
                t: rule.clone(),
                children,
            });
        }

        return Err(());
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
                production: vec![IN::AST("PAR".into())],
            },
            Rule {
                name: "PAR".into(),
                production: vec![
                    IN::Lexem("(".into()),
                    IN::AST("YEET".into()),
                    IN::Lexem(")".into()),
                ],
            },
            Rule {
                name: "YEET".into(),
                production: vec![IN::Lexem("yeet".into())],
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
                    production: vec![IN::AST("PAR".into())],
                },
                Rule {
                    name: "PAR".into(),
                    production: vec![
                        IN::Lexem("(".into()),
                        IN::Lexem("NUMBER".into()),
                        IN::Lexem(")".into()),
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
