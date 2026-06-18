# Resp - Master Architecture Rules

## Lexical Mapping (Esperanto → Rust)
| Esperanto | Rust     |
|-----------|----------|
| fk        | fn       |
| estu      | let      |
| ŝanĝ      | mut      |
| strk      | struct   |
| realigu   | impl     |
| trajto    | trait    |
| enumb     | enum     |
| kongruu   | match    |
| ripetu    | loop     |
| por       | for      |
| en        | in       |
| dum       | while    |
| se        | if       |
| alie      | else     |
| redonu    | return   |
| uzu       | use      |
| kesto     | crate    |
| malsekura | unsafe   |
| pub       | pub      |
| provu     | try      |
| kaptu     | catch    |
| etendu    | extends  |

## Type Mapping (Esperanto → Rust)
| Esperanto  | Rust     |
|------------|----------|
| ent8-128,g | i8-i128,isize |
| nat8-128,g | u8-u128,usize |
| glit32/64  | f32/f64  |
| bulea      | bool     |
| kar        | char     |
| ĉen        | str      |
| Teksto     | String   |
| Vektoro<T> | Vec<T>   |

## Architecture
- PEG grammar via pest in `resp-parser/grammar/esperanto.pest`
- AST in `resp-ast/src/lib.rs` (no external deps)
- Semantic analysis in `resp-sema/src/lib.rs` (SymbolTable + name mangling)
- Codegen via syn/quote in `resp-transpiler/src/lib.rs`
- CLI via clap in `resp-cli/src/main.rs`

## Critical Rules
1. SymbolTable MUST be populated BEFORE transpilation
2. Name mangling MUST resolve function overloading (e.g., `procesi_ent32_glit64`)
3. `etendu` inheritance MUST inject `__parent` field + `Deref`/`DerefMut` impls
4. `provu`/`kaptu` MUST transpile to `|| -> Result<_, Box<dyn Error>> { }()` with match
5. All `syn::Ident::new` MUST use real spans from `Located<T>`, NOT `call_site()`
6. CLI MUST capture rustc JSON errors and map to .resp source lines
7. `Kreis` and `Kreu` variants are REMOVED from AST
8. `Provu` uses `Box<Esprimo>` not `Vec<Deklaro>` for the try body
9. Parameters can have default values: `param = { identigilo ~ ":" ~ tipo ~ ("=" ~ expr)? }`

## Validation
- Always run `cargo check --workspace` after each phase
- Always run `cargo fmt` after writing .rs files
- Always run `cargo test --workspace` before declaring phase complete
