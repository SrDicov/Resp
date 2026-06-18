use resp_parser::{RespParser, parse_programo};
use pest::Parser;

fn print_pairs(pair: &pest::iterators::Pair<resp_parser::Rule>, depth: usize) {
    let indent = "  ".repeat(depth);
    eprintln!("{}{:?} = '{}'", indent, pair.as_rule(), pair.as_str().escape_debug());
    for child in pair.clone().into_inner() {
        print_pairs(&child, depth + 1);
    }
}

#[test]
fn debug_parse_simple() {
    let source = r#"uzu std::io;

fk ĉefa() {
    estu nomo: Teksto = "Mondo";
}
"#;
    let result = RespParser::parse(resp_parser::Rule::program, source);
    match result {
        Ok(pairs) => {
            for pair in pairs {
                print_pairs(&pair, 0);
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
    let prog = parse_programo(source).unwrap();
    eprintln!("Declarations: {}", prog.deklaroj.len());
    for d in &prog.deklaroj {
        eprintln!("  {:?}", d);
    }
}
