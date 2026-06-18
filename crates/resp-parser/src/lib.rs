//! Parser for the Resp programming language.
//!
//! Uses pest PEG grammar to parse `.resp` source files into
//! the AST defined in `resp-ast`.

use resp_ast::*;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser as PestDerive;

#[derive(PestDerive)]
#[grammar = "../grammar/esperanto.pest"]
pub struct RespParser;

pub fn parse_programo(eno: &str) -> Result<Programo, Error<Rule>> {
    let mut paroj = RespParser::parse(Rule::program, eno)?;
    let programo = paroj.next().unwrap();
    let mut deklaroj = Vec::new();

    for deklaro in programo.into_inner() {
        if let Some(d) = process_declaration(deklaro) {
            deklaroj.push(d);
        }
    }

    Ok(Programo { deklaroj })
}

fn span(el: &Pair<Rule>) -> Span {
    let (lineo, kolumno) = el.line_col();
    (lineo, kolumno)
}

fn process_declaration(pair: Pair<Rule>) -> Option<Deklaro> {
    let pair = if pair.as_rule() == Rule::declaration {
        pair.into_inner().next().unwrap()
    } else {
        pair
    };
    match pair.as_rule() {
        Rule::estu_decl => {
            let interno: Vec<_> = pair.into_inner().collect();
            let mut idx = 0;
            let mut mutebla = false;

            if idx < interno.len() && interno[idx].as_rule() == Rule::estu_keyword {
                idx += 1;
            }
            if idx < interno.len() && interno[idx].as_rule() == Rule::sang_keyword {
                mutebla = true;
                idx += 1;
            }

            let nomo = &interno[idx];
            idx += 1;
            let nomo_str = nomo.as_str().to_string();
            let sp = span(nomo);

            let tipo = if idx < interno.len() && interno[idx].as_rule() == Rule::tipo {
                let t = parse_tipo(interno[idx].clone());
                idx += 1;
                Some(t)
            } else {
                None
            };

            let expr = if idx < interno.len() {
                Some(parse_expr(interno[idx].clone()))
            } else {
                None
            };

            Some(Deklaro::Estu(Located::new(nomo_str, sp), tipo, expr, mutebla))
        }
        Rule::funkcio_decl => {
            let mut interno = pair.into_inner();
            let mut nomo_str = String::new();
            let mut sp = (0, 0);
            let mut parametroj = Vec::new();
            let mut reveno = None;
            let mut korpo = Vec::new();

            for child in &mut interno {
                match child.as_rule() {
                    Rule::identigilo => {
                        nomo_str = child.as_str().to_string();
                        sp = span(&child);
                    }
                    Rule::param_list => {
                        for param in child.into_inner() {
                            if param.as_rule() == Rule::param {
                                let mut p = param.into_inner();
                                let p_nomo = p.next().unwrap();
                                let p_tipo = p.next().unwrap();
                                let mut def_val = None;
                                if let Some(next) = p.next() {
                                    if next.as_rule() == Rule::expr {
                                        def_val = Some(parse_expr(next));
                                    }
                                }
                                parametroj.push(Parametro {
                                    nomo: Located::new(p_nomo.as_str().to_string(), span(&p_nomo)),
                                    tipo: Some(parse_tipo(p_tipo)),
                                    valor_defecto: def_val,
                                });
                            }
                        }
                    }
                    Rule::tipo => {
                        reveno = Some(parse_tipo(child));
                    }
                    Rule::bloko => {
                        for d in child.into_inner() {
                            if let Some(decl) = process_declaration(d) {
                                korpo.push(decl);
                            }
                        }
                    }
                    _ => {}
                }
            }

            Some(Deklaro::Funkcio {
                nomo: Located::new(nomo_str, sp),
                parametroj,
                reveno,
                korpo,
            })
        }
        Rule::strk_decl => {
            let mut interno = pair.into_inner();
            let nomo = interno.next().unwrap();
            let nomo_str = nomo.as_str().to_string();
            let sp = span(&nomo);

            let mut etendo = None;
            let mut kampoj = Vec::new();

            for child in interno {
                match child.as_rule() {
                    Rule::identigilo => {
                        etendo = Some(child.as_str().to_string());
                    }
                    Rule::kampo => {
                        let mut k = child.into_inner();
                        let vis = if k.peek().map_or(false, |p| p.as_rule() == Rule::pub_keyword)
                        {
                            k.next();
                            Some(Visiblo::Publika)
                        } else {
                            None
                        };
                        let k_nomo = k.next().unwrap();
                        let k_tipo = k.next().unwrap();
                        kampoj.push(Kampo {
                            nomo: Located::new(k_nomo.as_str().to_string(), span(&k_nomo)),
                            tipo: parse_tipo(k_tipo),
                            visiblo: vis,
                        });
                    }
                    _ => {}
                }
            }

            Some(Deklaro::Strk {
                nomo: Located::new(nomo_str, sp),
                etendo,
                kampoj,
            })
        }
        Rule::enumb_decl => {
            let mut interno = pair.into_inner();
            let nomo = interno.next().unwrap();
            let nomo_str = nomo.as_str().to_string();
            let sp = span(&nomo);

            let mut variantoj = Vec::new();
            for child in interno {
                if child.as_rule() == Rule::enumb_variant {
                    let mut v = child.into_inner();
                    let v_nomo = v.next().unwrap();
                    let mut tipoj = Vec::new();
                    for t in v {
                        tipoj.push(parse_tipo(t));
                    }
                    variantoj.push(Varianto {
                        nomo: Located::new(v_nomo.as_str().to_string(), span(&v_nomo)),
                        tipoj,
                    });
                }
            }

            Some(Deklaro::Enumb {
                nomo: Located::new(nomo_str, sp),
                variantoj,
            })
        }
        Rule::realigu_decl => {
            let mut interno = pair.into_inner();
            let tipo = interno.next().unwrap();
            let tipo_str = tipo.as_str().to_string();

            let mut membroj = Vec::new();
            for child in interno {
                if let Some(d) = process_declaration(child) {
                    membroj.push(d);
                }
            }

            Some(Deklaro::Realigu {
                tipo: tipo_str,
                membroj,
            })
        }
        Rule::trajto_decl => {
            let mut interno = pair.into_inner();
            let nomo = interno.next().unwrap();
            let nomo_str = nomo.as_str().to_string();
            let sp = span(&nomo);

            let mut metodoj = Vec::new();
            for child in interno {
                if let Some(d) = process_declaration(child) {
                    metodoj.push(d);
                }
            }

            Some(Deklaro::Trajto {
                nomo: Located::new(nomo_str, sp),
                metodoj,
            })
        }
        Rule::uzu_decl => {
            let vojo = pair.into_inner().skip(1).next().unwrap().as_str().to_string();
            Some(Deklaro::Uzu { vojo })
        }
        Rule::provu_stmt => {
            let interno: Vec<_> = pair.into_inner().collect();
            let mut idx = 0;
            if idx < interno.len() && interno[idx].as_rule() == Rule::provu_keyword {
                idx += 1;
            }
            let try_block = parse_block(interno[idx].clone());
            idx += 1;
            let catch_block = if idx < interno.len() && interno[idx].as_rule() == Rule::kaptu_keyword {
                idx += 1;
                Some(parse_block(interno[idx].clone()))
            } else {
                None
            };
            let expr = Esprimo::Provu(Box::new(Esprimo::Bloko(try_block)), catch_block);
            Some(Deklaro::EsprimoStmt(expr))
        }
        Rule::expr_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            Some(Deklaro::EsprimoStmt(expr))
        }
        Rule::se_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            Some(Deklaro::EsprimoStmt(expr))
        }
        Rule::dum_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            Some(Deklaro::EsprimoStmt(expr))
        }
        Rule::por_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            Some(Deklaro::EsprimoStmt(expr))
        }
        Rule::ripetu_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            Some(Deklaro::EsprimoStmt(expr))
        }
        Rule::kongruu_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            Some(Deklaro::EsprimoStmt(expr))
        }
        _ => None,
    }
}

fn parse_tipo(pair: Pair<Rule>) -> Tipo {
    match pair.as_rule() {
        Rule::tipo_ent => {
            let s = pair.as_str();
            match &s[3..] {
                "8" => Tipo::Ent8,
                "16" => Tipo::Ent16,
                "32" => Tipo::Ent32,
                "64" => Tipo::Ent64,
                "128" => Tipo::Ent128,
                "g" => Tipo::EntG,
                _ => unreachable!(),
            }
        }
        Rule::tipo_nat => {
            let s = pair.as_str();
            match &s[3..] {
                "8" => Tipo::Nat8,
                "16" => Tipo::Nat16,
                "32" => Tipo::Nat32,
                "64" => Tipo::Nat64,
                "128" => Tipo::Nat128,
                "g" => Tipo::NatG,
                _ => unreachable!(),
            }
        }
        Rule::tipo_glit => {
            let s = pair.as_str();
            match &s[4..] {
                "32" => Tipo::Glit32,
                "64" => Tipo::Glit64,
                _ => unreachable!(),
            }
        }
        Rule::tipo_bulea => Tipo::Bulea,
        Rule::tipo_kar => Tipo::Kar,
        Rule::tipo_cen => Tipo::Ĉen,
        Rule::tipo_teksto => Tipo::Teksto,
        Rule::tipo_vektoro => {
            let interno = pair.into_inner().next().unwrap();
            Tipo::Vektoro(Box::new(parse_tipo(interno)))
        }
        Rule::tipo => {
            let s = pair.as_str();
            if s.starts_with('&') {
                let mut interno = pair.into_inner();
                let first = interno.next().unwrap();
                let is_mut = first.as_rule() == Rule::sang_keyword;
                let t = if is_mut {
                    parse_tipo(interno.next().unwrap())
                } else {
                    parse_tipo(first)
                };
                if is_mut {
                    Tipo::ReferencoŜanĝ(Box::new(t))
                } else {
                    Tipo::Referenco(Box::new(t))
                }
            } else {
                let mut interno_iter = pair.into_inner();
                let first = interno_iter.next().unwrap();
                match first.as_rule() {
                    Rule::tipo_ent | Rule::tipo_nat | Rule::tipo_glit | Rule::tipo_bulea
                    | Rule::tipo_kar | Rule::tipo_cen | Rule::tipo_teksto | Rule::tipo_vektoro => {
                        parse_tipo(first)
                    }
                    Rule::identigilo => {
                        let name = first.as_str().to_string();
                        if let Some(list) = interno_iter.next() {
                            if list.as_rule() == Rule::tipo_list {
                                let generics: Vec<Tipo> = list.into_inner().map(parse_tipo).collect();
                                Tipo::Nombrita(name, generics)
                            } else {
                                Tipo::Malferma(name)
                            }
                        } else {
                            Tipo::Malferma(name)
                        }
                    }
                    _ => Tipo::Malferma(s.to_string()),
                }
            }
        }
        _ => Tipo::Malferma(pair.as_str().to_string()),
    }
}

fn parse_expr(pair: Pair<Rule>) -> Esprimo {
    match pair.as_rule() {
        Rule::expr => {
            let inner = pair.into_inner().next().unwrap();
            parse_expr(inner)
        }
        Rule::literal => {
            let lit = parse_literal(pair.clone());
            Esprimo::Literal(Located::new(lit, span(&pair)))
        }
        Rule::identigilo => {
            Esprimo::Identigilo(Located::new(pair.as_str().to_string(), span(&pair)))
        }
        Rule::asigno_expr => {
            let mut interno = pair.into_inner();
            let left = interno.next().unwrap();
            if let Some(right) = interno.next() {
                let right_expr = parse_expr(right);
                Esprimo::Asigno(
                    Box::new(parse_expr(left)),
                    Box::new(right_expr),
                )
            } else {
                parse_expr(left)
            }
        }
        Rule::or_expr => {
            let mut interno = pair.into_inner();
            let mut left = parse_expr(interno.next().unwrap());
            for op in interno {
                let right = parse_expr(op);
                left = Esprimo::Duarg(
                    Box::new(left),
                    Located::new(DuargOperaciilo::Au, (0, 0)),
                    Box::new(right),
                );
            }
            left
        }
        Rule::and_expr => {
            let mut interno = pair.into_inner();
            let mut left = parse_expr(interno.next().unwrap());
            for op in interno {
                let right = parse_expr(op);
                left = Esprimo::Duarg(
                    Box::new(left),
                    Located::new(DuargOperaciilo::Kaj, (0, 0)),
                    Box::new(right),
                );
            }
            left
        }
        Rule::cmp_expr => {
            let mut interno = pair.into_inner();
            let left = parse_expr(interno.next().unwrap());
            if let Some(op) = interno.next() {
                let op_str = op.as_str();
                let operaciilo = match op_str {
                    "==" => DuargOperaciilo::Egala,
                    "!=" => DuargOperaciilo::NeEgala,
                    "<" => DuargOperaciilo::Malpli,
                    "<=" => DuargOperaciilo::MalpliEgala,
                    ">" => DuargOperaciilo::Pli,
                    ">=" => DuargOperaciilo::PliEgala,
                    _ => unreachable!(),
                };
                let right = parse_expr(interno.next().unwrap());
                Esprimo::Duarg(
                    Box::new(left),
                    Located::new(operaciilo, span(&op)),
                    Box::new(right),
                )
            } else {
                left
            }
        }
        Rule::add_expr => {
            let mut interno = pair.into_inner();
            let mut left = parse_expr(interno.next().unwrap());
            while let Some(op) = interno.next() {
                let op_str = op.as_str();
                let operaciilo = match op_str {
                    "+" => DuargOperaciilo::Aldoni,
                    "-" => DuargOperaciilo::Subtrahi,
                    _ => unreachable!(),
                };
                let right = parse_expr(interno.next().unwrap());
                left = Esprimo::Duarg(
                    Box::new(left),
                    Located::new(operaciilo, (0, 0)),
                    Box::new(right),
                );
            }
            left
        }
        Rule::mul_expr => {
            let mut interno = pair.into_inner();
            let mut left = parse_expr(interno.next().unwrap());
            while let Some(op) = interno.next() {
                let op_str = op.as_str();
                let operaciilo = match op_str {
                    "*" => DuargOperaciilo::Multipliki,
                    "/" => DuargOperaciilo::Dividi,
                    "%" => DuargOperaciilo::Resto,
                    _ => unreachable!(),
                };
                let right = parse_expr(interno.next().unwrap());
                left = Esprimo::Duarg(
                    Box::new(left),
                    Located::new(operaciilo, (0, 0)),
                    Box::new(right),
                );
            }
            left
        }
        Rule::unary_expr => {
            let mut interno = pair.into_inner();
            let first = interno.next().unwrap();
            match first.as_rule() {
                Rule::unary_op => {
                    let op_str = first.as_str();
                    let expr = parse_expr(interno.next().unwrap());
                    let operaciilo = match op_str {
                        "-" => UnuargOperaciilo::Negi,
                        "!" => UnuargOperaciilo::Ne,
                        "&" => UnuargOperaciilo::Referenci,
                        "*" => UnuargOperaciilo::Dereferenci,
                        _ => unreachable!(),
                    };
                    Esprimo::Unuarg(Located::new(operaciilo, span(&first)), Box::new(expr))
                }
                Rule::voko_expr => parse_expr(first),
                _ => parse_expr(first),
            }
        }
        Rule::voko_expr => {
            let mut interno = pair.into_inner();
            let atomo = parse_expr(interno.next().unwrap());
            let mut result = atomo;
            for suf in interno {
                match suf.as_rule() {
                Rule::voko_sufikso => {
                    let suf_str = suf.as_str();
                    if suf_str == "?" {
                        result = Esprimo::Unuarg(
                            Located::new(UnuargOperaciilo::Demandilo, span(&suf)),
                            Box::new(result),
                        );
                        continue;
                    }
                    if suf_str.starts_with(".") {
                        let mut inner_iter = suf.into_inner();
                        let field = inner_iter.next().unwrap();
                        let field_str = field.as_str().to_string();
                        let field_span = span(&field);
                        // Check if this is a method call (has parentheses)
                        let is_method_call = suf_str.contains('(');
                        if is_method_call {
                            let mut args = Vec::new();
                            if let Some(args_pair) = inner_iter.next() {
                                if args_pair.as_rule() == Rule::expr_list {
                                    for expr in args_pair.into_inner() {
                                        args.push(parse_expr(expr));
                                    }
                                }
                            }
                            let member = Esprimo::Membro(
                                Box::new(result),
                                Located::new(field_str, field_span),
                            );
                            result = Esprimo::Voko(Box::new(member), args);
                        } else {
                            // Plain field access
                            result = Esprimo::Membro(
                                Box::new(result),
                                Located::new(field_str, field_span),
                            );
                        }
                        continue;
                    }
                    let is_macro = suf_str.starts_with('!');
                    let mut args = Vec::new();
                    for inner in suf.into_inner() {
                        if inner.as_rule() == Rule::expr_list {
                            for expr in inner.into_inner() {
                                args.push(parse_expr(expr));
                            }
                        } else {
                            args.push(parse_expr(inner));
                        }
                    }
                    if is_macro {
                        result = Esprimo::Makro(Box::new(result), args);
                    } else {
                        result = Esprimo::Voko(Box::new(result), args);
                    }
                }
                    _ => {}
                }
            }
            result
        }
        Rule::provu_expr => {
            let mut interno = pair.into_inner();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::provu_keyword) {
                interno.next();
            }
            let try_body = parse_block(interno.next().unwrap());
            let catch_block = if interno.peek().map_or(false, |p| p.as_rule() == Rule::kaptu_keyword) {
                interno.next();
                Some(parse_block(interno.next().unwrap()))
            } else {
                None
            };
            Esprimo::Provu(Box::new(Esprimo::Bloko(try_body)), catch_block)
        }
        Rule::se_expr => {
            let mut interno = pair.into_inner();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::se_keyword) {
                interno.next();
            }
            let kond = parse_expr(interno.next().unwrap());
            let tiam = parse_block(interno.next().unwrap());
            let alie = if interno.peek().map_or(false, |p| p.as_rule() == Rule::alie_keyword) {
                interno.next();
                Some(parse_block(interno.next().unwrap()))
            } else {
                None
            };
            Esprimo::Se(Box::new(kond), tiam, alie)
        }
        Rule::dum_expr => {
            let mut interno = pair.into_inner();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::dum_keyword) {
                interno.next();
            }
            let kond = parse_expr(interno.next().unwrap());
            let korpo = parse_block(interno.next().unwrap());
            Esprimo::Dum(Box::new(kond), korpo)
        }
        Rule::por_expr => {
            let mut interno = pair.into_inner();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::por_keyword) {
                interno.next();
            }
            let var = interno.next().unwrap();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::en_keyword) {
                interno.next();
            }
            let kolekto = parse_expr(interno.next().unwrap());
            let korpo = parse_block(interno.next().unwrap());
            Esprimo::Por(
                Located::new(var.as_str().to_string(), span(&var)),
                Box::new(kolekto),
                korpo,
            )
        }
        Rule::ripetu_expr => {
            let mut interno = pair.into_inner();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::ripetu_keyword) {
                interno.next();
            }
            let bloko = interno.next().unwrap();
            Esprimo::Ripetu(parse_block(bloko))
        }
        Rule::kongruu_expr => {
            let mut interno = pair.into_inner();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::kongruu_keyword) {
                interno.next();
            }
            let expr = parse_expr(interno.next().unwrap());
            let mut brakoj = Vec::new();
            for br in interno {
                let mut br_inner = br.into_inner();
                let patrono = parse_patrono(br_inner.next().unwrap());
                let korpo = parse_block(br_inner.next().unwrap());
                brakoj.push(Brako { patrono, korpo });
            }
            Esprimo::Kongruu(Box::new(expr), brakoj)
        }
        Rule::redonu_expr => {
            let mut interno = pair.into_inner();
            if interno.peek().map_or(false, |p| p.as_rule() == Rule::redonu_keyword) {
                interno.next();
            }
            let expr = interno.next().map(|e| parse_expr(e));
            Esprimo::Redonu(Box::new(expr.unwrap_or_else(|| {
                Esprimo::Bloko(Vec::new())
            })))
        }
        Rule::atomo => {
            let inner = pair.into_inner().next().unwrap();
            parse_expr(inner)
        }
        Rule::expr_list => {
            parse_expr(pair.into_inner().next().unwrap())
        }
        Rule::bloko => Esprimo::Bloko(parse_block(pair)),
        _ => Esprimo::Bloko(Vec::new()),
    }
}

fn parse_literal(pair: Pair<Rule>) -> Literal {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::integer_literal => {
            Literal::Ent(IntegerBase::Dekuma, inner.as_str().to_string())
        }
        Rule::float_literal => {
            Literal::Glit(inner.as_str().to_string())
        }
        Rule::string_literal => {
            let s = inner.as_str();
            Literal::Teksta(s[1..s.len() - 1].to_string())
        }
        Rule::char_literal => {
            let s = inner.as_str();
            let c = s.chars().nth(1).unwrap_or(' ');
            Literal::Kara(c)
        }
        Rule::bool_literal => {
            Literal::Bulea(inner.as_str() == "vera")
        }
        _ => unreachable!(),
    }
}

fn parse_patrono(pair: Pair<Rule>) -> Patrono {
    let inner = pair.into_inner().next().unwrap();
    let sp = span(&inner);
    match inner.as_rule() {
        Rule::literal => Patrono::Literal(Located::new(parse_literal(inner), sp)),
        Rule::identigilo => Patrono::Identigilo(Located::new(inner.as_str().to_string(), sp)),
        _ => Patrono::Ĉio,
    }
}

fn parse_block(pair: Pair<Rule>) -> Vec<Deklaro> {
    let mut deklaroj = Vec::new();
    for child in pair.into_inner() {
        if let Some(d) = process_declaration(child) {
            deklaroj.push(d);
        }
    }
    deklaroj
}
