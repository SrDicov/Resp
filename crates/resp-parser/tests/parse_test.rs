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

#[test]
fn parse_provu_kaptu() {
    let source = r#"fk test() {
    provu {
        println!("riskanta");
    } kaptu {
        println!("kaptita");
    }
}
"#;
    let prog = parse_programo(source).unwrap();
    assert_eq!(prog.deklaroj.len(), 1);
}

#[test]
fn parse_default_param() {
    let source = r#"fk saluton(nomo: Teksto = "Amiko") {
    println!("Saluton, {}!", nomo);
}
"#;
    let prog = parse_programo(source).unwrap();
    assert_eq!(prog.deklaroj.len(), 1);
}

#[test]
fn parse_etendu() {
    let source = r#"strk Ano {
    nomo: Teksto,
}

strk Administranto etendu Ano {
    administra_kodo: ent32,
}
"#;
    let prog = parse_programo(source).unwrap();
    assert_eq!(prog.deklaroj.len(), 2);
}

#[test]
fn parse_function_overload() {
    let source = r#"fk procesi(valoro: ent32) -> ent32 {
    redonu valoro;
}

fk procesi(valoro: entg) -> entg {
    redonu valoro;
}
"#;
    let prog = parse_programo(source).unwrap();
    assert_eq!(prog.deklaroj.len(), 2);
}

#[test]
fn parse_question_mark() {
    let source = r#"fk test() {
    estu x = io()?;
}
"#;
    let prog = parse_programo(source).unwrap();
    assert_eq!(prog.deklaroj.len(), 1);
}

#[test]
fn parse_generic_type() {
    let source = r#"fk test() -> Result<i32, String> {
    redonu Ok(42);
}
"#;
    let prog = parse_programo(source).unwrap();
    assert_eq!(prog.deklaroj.len(), 1);
}

#[test]
fn parse_method_call() {
    let source = r#"fk test() {
    estu x = "hello".to_string();
}
"#;
    let prog = parse_programo(source).unwrap();
    assert_eq!(prog.deklaroj.len(), 1);
}
