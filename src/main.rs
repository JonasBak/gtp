mod grammar;
mod parsing;

use grammar::*;

fn main() {
    env_logger::init();
    let g = get_parsing_grammar();
    let ast = g
        .parse(
            &r#"
            START     -> SUM;
            SUM       -> PRODUCT (OPA PRODUCT)*;
            PRODUCT   -> NUMBER (OPB NUMBER)*;
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
    println!("Grammar:\n{}\n\n{:?}", gp, gp);
    println!("{:#?}", gp.parse(&"1+2x3".into()));
    println!("{:#?}", gp.parse(&"1x2+3".into()));
}
