mod grammar;
mod parsing;

use grammar::*;

fn main() {
    env_logger::init();
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
    println!("AST:\n{:?}\n", ast);
    let gp = parse_ast_grammar(ast);
    println!("Grammar:\n{}\n\n{:?}", gp, gp);
    println!("{:?}", gp.parse(&"2*3".into()));
}
