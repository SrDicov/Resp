//! Semantic analyzer for the Resp programming language.
//!
//! Builds a `SymbolTable` by traversing the AST before transpilation.
//! Handles:
//! - Recording struct fields for `etendu` inheritance resolution
//! - Name mangling for function overloading

use resp_ast::*;
use std::collections::HashMap;

pub struct SymbolTable {
    pub structs: HashMap<String, StructInfo>,
    pub functions: HashMap<String, Vec<FuncInfo>>,
}

pub struct StructInfo {
    pub kampoj: Vec<Kampo>,
}

#[derive(Clone)]
pub struct FuncInfo {
    pub original_nomo: String,
    pub mangled_nomo: String,
    pub parametroj: Vec<Parametro>,
    pub reveno: Option<Tipo>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            structs: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_struct(&self, name: &str) -> Option<&StructInfo> {
        self.structs.get(name)
    }

    pub fn get_mangled(&self, name: &str, param_types: &[Tipo]) -> Option<&str> {
        self.functions.get(name).and_then(|funcs| {
            funcs.iter().find(|f| {
                f.parametroj.len() == param_types.len()
                    && f.parametroj.iter().zip(param_types).all(|(p, t)| {
                        p.tipo.as_ref().map_or(false, |pt| pt == t)
                    })
            }).map(|f| f.mangled_nomo.as_str())
        })
    }
}

pub fn analyze_program(programo: &mut Programo) -> SymbolTable {
    let mut tabla = SymbolTable::new();

    // First pass: collect struct definitions and function signatures
    for deklaro in &programo.deklaroj {
        match deklaro {
            Deklaro::Strk { nomo, kampoj, .. } => {
                tabla.structs.insert(nomo.value.clone(), StructInfo {
                    kampoj: kampoj.clone(),
                });
            }
            Deklaro::Funkcio { nomo, parametroj, reveno, .. } => {
                let entry = tabla.functions.entry(nomo.value.clone()).or_default();
                entry.push(FuncInfo {
                    original_nomo: nomo.value.clone(),
                    mangled_nomo: nomo.value.clone(),
                    parametroj: parametroj.clone(),
                    reveno: reveno.clone(),
                });
            }
            _ => {}
        }
    }

    // Second pass: perform name mangling where collisions exist
    for (_name, funcs) in tabla.functions.iter_mut() {
        if funcs.len() > 1 {
            for func in funcs.iter_mut() {
                let suffix: Vec<String> = func.parametroj.iter()
                    .filter_map(|p| p.tipo.as_ref())
                    .map(|t| tipo_to_mangle_suffix(t))
                    .collect();
                let suffix_str = if suffix.is_empty() { "".to_string() } else { format!("_{}", suffix.join("_")) };
                func.mangled_nomo = format!("{}{}", func.original_nomo, suffix_str);
            }
            // Apply mangling to AST
            for deklaro in &mut programo.deklaroj {
                if let Deklaro::Funkcio { nomo, parametroj, .. } = deklaro {
                    if let Some(matched) = funcs.iter().find(|f| {
                        f.original_nomo == nomo.value
                            && f.parametroj.len() == parametroj.len()
                            && f.parametroj.iter().zip(parametroj.iter()).all(|(a, b)| {
                                a.tipo == b.tipo
                            })
                    }) {
                        nomo.value = matched.mangled_nomo.clone();
                    }
                }
            }
        }
    }

    tabla
}

fn tipo_to_mangle_suffix(t: &Tipo) -> String {
    match t {
        Tipo::Ent8 => "i8",
        Tipo::Ent16 => "i16",
        Tipo::Ent32 => "i32",
        Tipo::Ent64 => "i64",
        Tipo::Ent128 => "i128",
        Tipo::EntG => "isize",
        Tipo::Nat8 => "u8",
        Tipo::Nat16 => "u16",
        Tipo::Nat32 => "u32",
        Tipo::Nat64 => "u64",
        Tipo::Nat128 => "u128",
        Tipo::NatG => "usize",
        Tipo::Glit32 => "f32",
        Tipo::Glit64 => "f64",
        Tipo::Bulea => "bool",
        Tipo::Kar => "char",
        Tipo::Ĉen => "str",
        Tipo::Teksto => "String",
        Tipo::Vektoro(_) => "Vec",
        Tipo::Nombrita(n, _) => return n.to_lowercase(),
        Tipo::Referenco(t) => return format!("ref_{}", tipo_to_mangle_suffix(t)),
        Tipo::ReferencoŜanĝ(t) => return format!("refmut_{}", tipo_to_mangle_suffix(t)),
        Tipo::Malferma(n) => return n.to_lowercase(),
    }
    .to_string()
}
