mod grammar;
mod parsing;

use grammar::*;

fn main() {
    env_logger::init();
    let g = get_parsing_grammar();
    println!(
        "{:?}",
        g.parse(
            &r#"
            START     -> PRODUCT;
            SUM       -> PRODUCT (OPA SUM)*;
            PRODUCT   -> NUMBER (OPB PRODUCT)*;
            NUMBER    -> num;
            NUMBER    -> minus num;

            OPA       -> pluss;
            OPA       -> minus;
            OPB       -> multiply;
            OPB       -> divide;

            >pluss    -> '+';
            >minus    -> '-';
            >multiply -> 'x';
            >divide   -> '/';
            >num      -> '\d+';
            "#
            .into()
        )
    );
}
