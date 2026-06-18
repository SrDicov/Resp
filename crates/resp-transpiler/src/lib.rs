//! Transpiler from Resp AST to Rust source code.
//!
//! Uses `syn` and `quote` to generate Rust token streams
//! from the AST defined in `resp-ast`. Uses `resp-sema` for
//! semantic analysis (SymbolTable, name mangling).

use proc_macro2::TokenStream;
use quote::quote;
use resp_ast::*;
use resp_sema::SymbolTable;

pub fn transpile(programo: &Programo) -> TokenStream {
    let mut tokens = TokenStream::new();
    for deklaro in &programo.deklaroj {
        tokens.extend(transpile_deklaro(deklaro, None));
    }
    tokens
}

pub fn transpile_with_sema(programo: &Programo, tabla: &SymbolTable) -> TokenStream {
    let mut tokens = TokenStream::new();
    for deklaro in &programo.deklaroj {
        tokens.extend(transpile_deklaro(deklaro, Some(tabla)));
    }
    tokens
}

fn map_nomo(s: &str) -> String {
    match s {
        "ĉefa" => "main".to_string(),
        _ => s.to_string(),
    }
}

fn transpile_deklaro(deklaro: &Deklaro, tabla: Option<&SymbolTable>) -> TokenStream {
    match deklaro {
        Deklaro::Uzu { vojo } => {
            let parts: Vec<&str> = vojo.split("::").collect();
            if parts.last() == Some(&"*") {
                let idents: Vec<syn::Ident> = parts[..parts.len() - 1].iter()
                    .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
                    .collect();
                let last = &idents[..];
                quote! { use #(#last)::*; }
            } else {
                let idents: Vec<syn::Ident> = parts.iter()
                    .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
                    .collect();
                let first = &idents[0];
                let rest = &idents[1..];
                quote! { use #first #(::#rest)*; }
            }
        }
        Deklaro::Estu(nomo, tipo, expr, mutebla) => {
            let nomo_ident = syn::Ident::new(&nomo.value, proc_macro2::Span::call_site());
            let tip = tipo.as_ref().map(map_tipo);
            let mut ekspr = expr.as_ref().map(|e| transpile_expr(e));
            if let (Some(&Tipo::Teksto), Some(e)) = (tipo.as_ref(), &ekspr) {
                let e_str = e.to_string();
                if e_str.starts_with('"') {
                    let inner = e_str.trim_matches('"');
                    let lit = proc_macro2::Literal::string(inner);
                    ekspr = Some(quote! { #lit.to_string() });
                }
            }
            if *mutebla {
                match (tip, ekspr) {
                    (Some(t), Some(e)) => quote! { let mut #nomo_ident: #t = #e; },
                    (Some(t), None) => quote! { let mut #nomo_ident: #t = Default::default(); },
                    (None, Some(e)) => quote! { let mut #nomo_ident = #e; },
                    (None, None) => quote! { let mut #nomo_ident = (); },
                }
            } else {
                match (tip, ekspr) {
                    (Some(t), Some(e)) => quote! { let #nomo_ident: #t = #e; },
                    (Some(t), None) => quote! { let #nomo_ident: #t = Default::default(); },
                    (None, Some(e)) => quote! { let #nomo_ident = #e; },
                    (None, None) => quote! { let #nomo_ident = (); },
                }
            }
        }
        Deklaro::Funkcio { nomo, parametroj, reveno, korpo } => {
            let nomo_ident = syn::Ident::new(&map_nomo(&nomo.value), proc_macro2::Span::call_site());
            let params: Vec<_> = parametroj.iter().map(|p| {
                let p_ident = syn::Ident::new(&p.nomo.value, proc_macro2::Span::call_site());
                let p_tipo = p.tipo.as_ref().map(map_tipo);
                match (&p.valor_defecto, &p_tipo) {
                    (Some(def), Some(t)) => {
                        let d = transpile_expr(def);
                        quote! { #p_ident: #t = #d }
                    }
                    (Some(def), None) => {
                        let d = transpile_expr(def);
                        quote! { #p_ident = #d }
                    }
                    (None, Some(t)) => quote! { #p_ident: #t },
                    (None, None) => quote! { #p_ident },
                }
            }).collect();
            let ret = reveno.as_ref().map(map_tipo);
            let korpo_tokens: Vec<_> = korpo.iter().map(|d| transpile_deklaro(d, tabla)).collect();
            if let Some(r) = &ret {
                quote! {
                    fn #nomo_ident(#(#params),*) -> #r {
                        #(#korpo_tokens)*
                    }
                }
            } else {
                quote! {
                    fn #nomo_ident(#(#params),*) {
                        #(#korpo_tokens)*
                    }
                }
            }
        }
        Deklaro::Strk { nomo, etendo, kampoj } => {
            let nomo_ident = syn::Ident::new(&nomo.value, proc_macro2::Span::call_site());
            let field_tokens: Vec<_> = kampoj.iter().map(|k| {
                let k_ident = syn::Ident::new(&k.nomo.value, proc_macro2::Span::call_site());
                let k_tipo = map_tipo(&k.tipo);
                let pub_attr = match k.visiblo {
                    Some(Visiblo::Publika) => quote! { pub },
                    _ => quote! {},
                };
                quote! { #pub_attr #k_ident: #k_tipo }
            }).collect();

            let mut tokens = TokenStream::new();

            match etendo {
                Some(parent_name) => {
                    let parent_ident = syn::Ident::new(parent_name, proc_macro2::Span::call_site());
                    // Check SymbolTable for parent struct fields
                    let parent_fields = tabla.and_then(|t| t.get_struct(parent_name));
                    let parent_field_tokens: Vec<_> = parent_fields.map(|s| {
                        s.kampoj.iter().map(|k| {
                            let k_ident = syn::Ident::new(&k.nomo.value, proc_macro2::Span::call_site());
                            let k_tipo = map_tipo(&k.tipo);
                            let pub_attr = match k.visiblo {
                                Some(Visiblo::Publika) => quote! { pub },
                                _ => quote! {},
                            };
                            quote! { #pub_attr #k_ident: #k_tipo }
                        }).collect::<Vec<_>>()
                    }).unwrap_or_default();

                    // Generate struct with all fields + __parent
                    let struct_def = quote! {
                        struct #nomo_ident {
                            #(#parent_field_tokens,)*
                            #(#field_tokens),*
                        }
                    };
                    tokens.extend(struct_def);

                    // Generate Deref impl
                    let deref_impl = quote! {
                        impl std::ops::Deref for #nomo_ident {
                            type Target = #parent_ident;
                            fn deref(&self) -> &Self::Target {
                                unsafe { std::mem::transmute::<&#nomo_ident, &Self::Target>(self) }
                            }
                        }
                    };
                    tokens.extend(deref_impl);

                    // Generate DerefMut impl
                    let deref_mut_impl = quote! {
                        impl std::ops::DerefMut for #nomo_ident {
                            fn deref_mut(&mut self) -> &mut Self::Target {
                                unsafe { std::mem::transmute::<&mut #nomo_ident, &mut Self::Target>(self) }
                            }
                        }
                    };
                    tokens.extend(deref_mut_impl);
                }
                None => {
                    tokens.extend(quote! {
                        struct #nomo_ident {
                            #(#field_tokens),*
                        }
                    });
                }
            }
            tokens
        }
        Deklaro::Enumb { nomo, variantoj } => {
            let nomo_ident = syn::Ident::new(&nomo.value, proc_macro2::Span::call_site());
            let variant_tokens: Vec<_> = variantoj.iter().map(|v| {
                let v_ident = syn::Ident::new(&v.nomo.value, proc_macro2::Span::call_site());
                let v_tipoj: Vec<_> = v.tipoj.iter().map(map_tipo).collect();
                if v_tipoj.is_empty() {
                    quote! { #v_ident }
                } else {
                    quote! { #v_ident(#(#v_tipoj),*) }
                }
            }).collect();
            quote! {
                enum #nomo_ident {
                    #(#variant_tokens),*
                }
            }
        }
        Deklaro::Realigu { tipo, membroj } => {
            let tipo_ident: syn::Ident = syn::Ident::new(tipo, proc_macro2::Span::call_site());
            let membroj_tokens: Vec<_> = membroj.iter().map(|d| transpile_deklaro(d, tabla)).collect();
            quote! {
                impl #tipo_ident {
                    #(#membroj_tokens)*
                }
            }
        }
        Deklaro::Trajto { nomo, metodoj } => {
            let nomo_ident = syn::Ident::new(&nomo.value, proc_macro2::Span::call_site());
            let metodoj_tokens: Vec<_> = metodoj.iter().map(|d| {
                if let Deklaro::Funkcio { nomo, parametroj, reveno, .. } = d {
                    let nom_ident = syn::Ident::new(&nomo.value, proc_macro2::Span::call_site());
                    let params: Vec<_> = parametroj.iter().map(|p| {
                        let p_ident = syn::Ident::new(&p.nomo.value, proc_macro2::Span::call_site());
                        let p_tipo = p.tipo.as_ref().map(map_tipo);
                        quote! { #p_ident: #p_tipo }
                    }).collect();
                    let ret = reveno.as_ref().map(map_tipo);
                    if let Some(r) = &ret {
                        quote! { fn #nom_ident(#(#params)*) -> #r; }
                    } else {
                        quote! { fn #nom_ident(#(#params)*); }
                    }
                } else {
                    TokenStream::new()
                }
            }).collect();
            quote! {
                trait #nomo_ident {
                    #(#metodoj_tokens)*
                }
            }
        }
        Deklaro::EsprimoStmt(expr) => {
            let e = transpile_expr(expr);
            match expr {
                Esprimo::Se(..) | Esprimo::Dum(..) | Esprimo::Por(..)
                    | Esprimo::Ripetu(..) | Esprimo::Kongruu(..)
                    | Esprimo::Provu(..) => quote! { #e },
                _ => quote! { #e; },
            }
        }
        Deklaro::Komento(_) => TokenStream::new(),
    }
}

fn transpile_expr(expr: &Esprimo) -> TokenStream {
    match expr {
        Esprimo::Literal(lit) => {
            match &lit.value {
                Literal::Ent(_, v) => {
                    let n: syn::LitInt = syn::parse_str(v).unwrap();
                    quote! { #n }
                }
                Literal::Glit(v) => {
                    let n: syn::LitFloat = syn::parse_str(v).unwrap();
                    quote! { #n }
                }
                Literal::Teksta(v) => {
                    let lit = proc_macro2::Literal::string(v);
                    quote! { #lit }
                }
                Literal::Kara(c) => {
                    let lit = proc_macro2::Literal::character(*c);
                    quote! { #lit }
                }
                Literal::Bulea(v) => {
                    if *v { quote! { true } } else { quote! { false } }
                }
            }
        }
        Esprimo::Identigilo(id) => {
            let ident = syn::Ident::new(&id.value, proc_macro2::Span::call_site());
            quote! { #ident }
        }
        Esprimo::Unuarg(op, expr) => {
            let e = transpile_expr(expr);
            match &op.value {
                UnuargOperaciilo::Negi => quote! { -#e },
                UnuargOperaciilo::Ne => quote! { !#e },
                UnuargOperaciilo::Referenci => quote! { &#e },
                UnuargOperaciilo::Dereferenci => quote! { *#e },
                UnuargOperaciilo::Demandilo => quote! { #e? },
            }
        }
        Esprimo::Duarg(left, op, right) => {
            let l = transpile_expr(left);
            let r = transpile_expr(right);
            let op_str = match &op.value {
                DuargOperaciilo::Aldoni => quote! { + },
                DuargOperaciilo::Subtrahi => quote! { - },
                DuargOperaciilo::Multipliki => quote! { * },
                DuargOperaciilo::Dividi => quote! { / },
                DuargOperaciilo::Resto => quote! { % },
                DuargOperaciilo::Egala => quote! { == },
                DuargOperaciilo::NeEgala => quote! { != },
                DuargOperaciilo::Malpli => quote! { < },
                DuargOperaciilo::MalpliEgala => quote! { <= },
                DuargOperaciilo::Pli => quote! { > },
                DuargOperaciilo::PliEgala => quote! { >= },
                DuargOperaciilo::Kaj => quote! { && },
                DuargOperaciilo::Au => quote! { || },
            };
            quote! { #l #op_str #r }
        }
        Esprimo::Asigno(left, right) => {
            let l = transpile_expr(left);
            let r = transpile_expr(right);
            quote! { #l = #r }
        }
        Esprimo::Voko(func, args) => {
            let f = transpile_expr(func);
            let a: Vec<_> = args.iter().map(transpile_expr).collect();
            quote! { #f(#(#a),*) }
        }
        Esprimo::Makro(func, args) => {
            let f = transpile_expr(func);
            let a: Vec<_> = args.iter().map(transpile_expr).collect();
            quote! { #f!(#(#a),*) }
        }
        Esprimo::Membro(obj, field) => {
            let o = transpile_expr(obj);
            let f = syn::Ident::new(&field.value, proc_macro2::Span::call_site());
            quote! { #o.#f }
        }
        Esprimo::Indekso(arr, idx) => {
            let a = transpile_expr(arr);
            let i = transpile_expr(idx);
            quote! { #a[#i] }
        }
        Esprimo::Bloko(deklaroj) => {
            let d: Vec<_> = deklaroj.iter().map(|d| transpile_deklaro(d, None)).collect();
            quote! { { #(#d)* } }
        }
        Esprimo::Se(kond, tiam, alie) => {
            let k = transpile_expr(kond);
            let t: Vec<_> = tiam.iter().map(|d| transpile_deklaro(d, None)).collect();
            match alie {
                Some(a) => {
                    let a_tokens: Vec<_> = a.iter().map(|d| transpile_deklaro(d, None)).collect();
                    quote! { if #k { #(#t)* } else { #(#a_tokens)* } }
                }
                None => quote! { if #k { #(#t)* } },
            }
        }
        Esprimo::Dum(kond, korpo) => {
            let k = transpile_expr(kond);
            let k_tokens: Vec<_> = korpo.iter().map(|d| transpile_deklaro(d, None)).collect();
            quote! { while #k { #(#k_tokens)* } }
        }
        Esprimo::Por(var, kolekto, korpo) => {
            let v = syn::Ident::new(&var.value, proc_macro2::Span::call_site());
            let k = transpile_expr(kolekto);
            let k_tokens: Vec<_> = korpo.iter().map(|d| transpile_deklaro(d, None)).collect();
            quote! { for #v in #k { #(#k_tokens)* } }
        }
        Esprimo::Ripetu(korpo) => {
            let k_tokens: Vec<_> = korpo.iter().map(|d| transpile_deklaro(d, None)).collect();
            quote! { loop { #(#k_tokens)* } }
        }
        Esprimo::Kongruu(expr, brakoj) => {
            let e = transpile_expr(expr);
            let b: Vec<_> = brakoj.iter().map(|br| {
                let patrono = transpile_patrono(&br.patrono);
                let korpo: Vec<_> = br.korpo.iter().map(|d| transpile_deklaro(d, None)).collect();
                quote! { #patrono => { #(#korpo)* } }
            }).collect();
            quote! { match #e { #(#b)* } }
        }
        Esprimo::Redonu(expr) => {
            let e = transpile_expr(expr);
            quote! { return #e }
        }
        Esprimo::Provu(try_expr, catch_block) => {
            let try_tokens = transpile_expr(try_expr);
            match catch_block {
                Some(catch) => {
                    let catch_tokens: Vec<_> = catch.iter().map(|d| transpile_deklaro(d, None)).collect();
                    quote! {
                        (|| -> std::result::Result<_, Box<dyn std::error::Error>> {
                            Ok({ #try_tokens })
                        })().unwrap_or_else(|_| { #(#catch_tokens)* })
                    }
                }
                None => {
                    quote! {
                        (|| -> std::result::Result<_, Box<dyn std::error::Error>> {
                            Ok({ #try_tokens })
                        })().unwrap()
                    }
                }
            }
        }
    }
}

fn transpile_patrono(patrono: &Patrono) -> TokenStream {
    match patrono {
        Patrono::Ĉio => quote! { _ },
        Patrono::Literal(lit) => transpile_expr(&Esprimo::Literal(lit.clone())),
        Patrono::Identigilo(id) => {
            let ident = syn::Ident::new(&id.value, proc_macro2::Span::call_site());
            quote! { #ident }
        }
        Patrono::Varianto(name, args) => {
            let ident = syn::Ident::new(&name.value, proc_macro2::Span::call_site());
            match args {
                Some(a) => {
                    let a_tokens: Vec<_> = a.iter().map(transpile_patrono).collect();
                    quote! { #ident(#(#a_tokens),*) }
                }
                None => quote! { #ident },
            }
        }
        Patrono::Kondiĉa(p, _) => transpile_patrono(p),
    }
}

fn map_tipo(tipo: &Tipo) -> TokenStream {
    match tipo {
        Tipo::Ent8 => quote! { i8 },
        Tipo::Ent16 => quote! { i16 },
        Tipo::Ent32 => quote! { i32 },
        Tipo::Ent64 => quote! { i64 },
        Tipo::Ent128 => quote! { i128 },
        Tipo::EntG => quote! { isize },
        Tipo::Nat8 => quote! { u8 },
        Tipo::Nat16 => quote! { u16 },
        Tipo::Nat32 => quote! { u32 },
        Tipo::Nat64 => quote! { u64 },
        Tipo::Nat128 => quote! { u128 },
        Tipo::NatG => quote! { usize },
        Tipo::Glit32 => quote! { f32 },
        Tipo::Glit64 => quote! { f64 },
        Tipo::Bulea => quote! { bool },
        Tipo::Kar => quote! { char },
        Tipo::Ĉen => quote! { str },
        Tipo::Teksto => quote! { String },
        Tipo::Vektoro(t) => {
            let inner = map_tipo(t);
            quote! { Vec<#inner> }
        }
        Tipo::Nombrita(n, generics) => {
            let ident = syn::Ident::new(n, proc_macro2::Span::call_site());
            if generics.is_empty() {
                quote! { #ident }
            } else {
                let g: Vec<_> = generics.iter().map(map_tipo).collect();
                quote! { #ident<#(#g),*> }
            }
        }
        Tipo::Referenco(t) => {
            let inner = map_tipo(t).to_string();
            format!("&{}", inner).parse().unwrap()
        }
        Tipo::ReferencoŜanĝ(t) => {
            let inner = map_tipo(t).to_string();
            format!("&mut {}", inner).parse().unwrap()
        }
        Tipo::Malferma(n) => {
            let ident = syn::Ident::new(n, proc_macro2::Span::call_site());
            quote! { #ident }
        }
    }
}
