# [WIP] generic text parser

> Tool to parse a text based on a grammar, and output the abstract syntax tree in a structured format (like json)

## Example
```bash
$ > grammar << EOF
START     -> ( SUM )
SUM       -> ( PRODUCT ( OPA PRODUCT )* )
PRODUCT   -> ( NUMBER ( OPB NUMBER )* )
NUMBER    -> ( num )
NUMBER    -> ( minus num )
OPA       -> ( ( pluss ) | ( minus ) )
OPB       -> ( ( multiply ) | ( divide ) )

>pluss    -> '\+'
>minus    -> '-'
>multiply -> 'x'
>divide   -> '/'
>num      -> '\d+'
EOF

$ cargo run -- grammar
START -> ( SUM )
SUM -> ( PRODUCT ( OPA PRODUCT )* )
PRODUCT -> ( NUMBER ( OPB NUMBER )* )
NUMBER -> ( num )
NUMBER -> ( minus num )
OPA -> ( ( pluss ) | ( minus ) )
OPB -> ( ( multiply ) | ( divide ) )

>pluss -> '\+'
>minus -> '-'
>multiply -> 'x'
>divide -> '/'
>num -> '\d+'

$ cargo run -- grammar "1+2x1+3" -o yml --bubble
---
type: SUM
children:
  - type: num
    raw: "1"
  - type: pluss
    raw: +
  - type: PRODUCT
    children:
      - type: num
        raw: "2"
      - type: multiply
        raw: x
      - type: num
        raw: "1"
  - type: pluss
    raw: +
  - type: num
    raw: "3"

$ echo '++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.' > hello_world
$ cargo run --example brainfuck hello_world
Hello World!

```
