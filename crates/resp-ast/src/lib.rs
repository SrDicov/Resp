//! AST-types for the Resp programming language.
//!
//! This crate defines the abstract syntax tree (AST) used by the Resp
//! transpiler pipeline. It has no external dependencies.

use std::fmt;

pub type Span = (usize, usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Located<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Located<T> {
    pub fn new(value: T, span: Span) -> Self {
        Located { value, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tipo {
    Ent8,
    Ent16,
    Ent32,
    Ent64,
    Ent128,
    EntG,
    Nat8,
    Nat16,
    Nat32,
    Nat64,
    Nat128,
    NatG,
    Glit32,
    Glit64,
    Bulea,
    Kar,
    Ĉen,
    Teksto,
    Vektoro(Box<Tipo>),
    Nombrita(String, Vec<Tipo>),
    Referenco(Box<Tipo>),
    ReferencoŜanĝ(Box<Tipo>),
    Malferma(String),
}

impl fmt::Display for Tipo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tipo::Ent8 => write!(f, "ent8"),
            Tipo::Ent16 => write!(f, "ent16"),
            Tipo::Ent32 => write!(f, "ent32"),
            Tipo::Ent64 => write!(f, "ent64"),
            Tipo::Ent128 => write!(f, "ent128"),
            Tipo::EntG => write!(f, "entg"),
            Tipo::Nat8 => write!(f, "nat8"),
            Tipo::Nat16 => write!(f, "nat16"),
            Tipo::Nat32 => write!(f, "nat32"),
            Tipo::Nat64 => write!(f, "nat64"),
            Tipo::Nat128 => write!(f, "nat128"),
            Tipo::NatG => write!(f, "natg"),
            Tipo::Glit32 => write!(f, "glit32"),
            Tipo::Glit64 => write!(f, "glit64"),
            Tipo::Bulea => write!(f, "bulea"),
            Tipo::Kar => write!(f, "kar"),
            Tipo::Ĉen => write!(f, "ĉen"),
            Tipo::Teksto => write!(f, "Teksto"),
            Tipo::Vektoro(t) => write!(f, "Vektoro<{}>", t),
            Tipo::Nombrita(n, _) => write!(f, "{}", n),
            Tipo::Referenco(t) => write!(f, "&{}", t),
            Tipo::ReferencoŜanĝ(t) => write!(f, "&ŝanĝ {}", t),
            Tipo::Malferma(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Ent(IntegerBase, String),
    Glit(String),
    Teksta(String),
    Kara(char),
    Bulea(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntegerBase {
    Dekuma,
    Duuma,
    Okuma,
    Deksesuma,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnuargOperaciilo {
    Negi,
    Ne,
    Referenci,
    Dereferenci,
    Demandilo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DuargOperaciilo {
    Aldoni,
    Subtrahi,
    Multipliki,
    Dividi,
    Resto,
    Egala,
    NeEgala,
    Malpli,
    MalpliEgala,
    Pli,
    PliEgala,
    Kaj,
    Au,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Esprimo {
    Literal(Located<Literal>),
    Identigilo(Located<String>),
    Unuarg(Located<UnuargOperaciilo>, Box<Esprimo>),
    Duarg(Box<Esprimo>, Located<DuargOperaciilo>, Box<Esprimo>),
    Asigno(Box<Esprimo>, Box<Esprimo>),
    Voko(Box<Esprimo>, Vec<Esprimo>),
    Makro(Box<Esprimo>, Vec<Esprimo>),
    Membro(Box<Esprimo>, Located<String>),
    Indekso(Box<Esprimo>, Box<Esprimo>),
    Bloko(Vec<Deklaro>),
    Se(Box<Esprimo>, Vec<Deklaro>, Option<Vec<Deklaro>>),
    Dum(Box<Esprimo>, Vec<Deklaro>),
    Por(Located<String>, Box<Esprimo>, Vec<Deklaro>),
    Ripetu(Vec<Deklaro>),
    Kongruu(Box<Esprimo>, Vec<Brako>),
    Redonu(Box<Esprimo>),
    Provu(Box<Esprimo>, Option<Vec<Deklaro>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Brako {
    pub patrono: Patrono,
    pub korpo: Vec<Deklaro>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Patrono {
    Ĉio,
    Literal(Located<Literal>),
    Identigilo(Located<String>),
    Varianto(Located<String>, Option<Vec<Patrono>>),
    Kondiĉa(Box<Patrono>, Box<Esprimo>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Deklaro {
    Estu(Located<String>, Option<Tipo>, Option<Esprimo>, bool),
    Funkcio {
        nomo: Located<String>,
        parametroj: Vec<Parametro>,
        reveno: Option<Tipo>,
        korpo: Vec<Deklaro>,
    },
    Strk {
        nomo: Located<String>,
        etendo: Option<String>,
        kampoj: Vec<Kampo>,
    },
    Enumb {
        nomo: Located<String>,
        variantoj: Vec<Varianto>,
    },
    Realigu {
        tipo: String,
        membroj: Vec<Deklaro>,
    },
    Trajto {
        nomo: Located<String>,
        metodoj: Vec<Deklaro>,
    },
    Uzu {
        vojo: String,
    },
    EsprimoStmt(Esprimo),
    Komento(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parametro {
    pub nomo: Located<String>,
    pub tipo: Option<Tipo>,
    pub valor_defecto: Option<Esprimo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Kampo {
    pub nomo: Located<String>,
    pub tipo: Tipo,
    pub visiblo: Option<Visiblo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Varianto {
    pub nomo: Located<String>,
    pub tipoj: Vec<Tipo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visiblo {
    Publika,
    Privata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Programo {
    pub deklaroj: Vec<Deklaro>,
}
