# Resp - Programlingvo Esperanto → Rust

**Resp** estas programlingvo, kiu transpilas Esperantan kodarbon al Rust. Ĝi kombinas la ergonomion de C++ kun la sekureco de Rust, uzante vortprovizon el Esperanto.

```
fk fibonacco(n: ent32) -> ent32 {
    se n <= 1 {
        redonu n;
    }
    redonu fibonacco(n - 1) + fibonacco(n - 2);
}

fk ĉefa() {
    estu rezulto: ent32 = fibonacco(10);
    println!("Fibonacci(10) = {}", rezulto);
}
```

## Instalado

### Antaŭkondiĉoj

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

### Konstrui

```bash
git clone https://github.com/SrDicov/Resp.git
cd Resp
cargo build --release
```

### Uzado

```bash
# Transpili .resp dosieron al Rust
cargo run --bin resp-cli -- programo.resp -o programo.rs

# Aŭtomate formati kaj kompili
cargo run --bin resp-cli -- programo.resp -o programo.rs --fmt --compile
```

## Ekzemploj

```bash
# Saluton, Mondo!
cargo run --bin resp-cli -- examples/hello.resp -o /tmp/hello.rs --fmt --compile
/tmp/hello
# => Saluton, Mondo!

# Fibonacco
cargo run --bin resp-cli -- examples/fibonacci.resp -o /tmp/fib.rs --fmt --compile
/tmp/fib
# => Fibonacci(10) = 55
```

## Ŝlosilvortoj

| Esperanto | Rust    | Priskribo                         |
|-----------|---------|-----------------------------------|
| `fk`      | `fn`    | Funkcia deklaracio                |
| `estu`    | `let`   | Variabla deklaracio               |
| `ŝanĝ`    | `mut`   | Mutabileco                        |
| `strk`    | `struct`| Struktura difino                  |
| `realigu` | `impl`  | Efektivigo de metodoj             |
| `trajto`  | `trait` | Interfaca difino                  |
| `enumb`   | `enum`  | Nombritaj tipoj                   |
| `kongruu` | `match` | Padrona kongruigo                 |
| `ripetu`  | `loop`  | Senfina buklo                     |
| `por`     | `for`   | Iteracio super kolekto            |
| `en`      | `in`    | Enkluzivo en `por` buklo          |
| `dum`     | `while` | Kondiĉa buklo                     |
| `se`      | `if`    | Kondiĉa branĉo                    |
| `alie`    | `else`  | Nega kondiĉa branĉo               |
| `redonu`  | `return`| Reveno de valoro                  |
| `uzu`     | `use`   | Importo de moduloj                |
| `kesto`   | `crate` | Radiko de paketo                  |
| `malsekura`| `unsafe`| Nersekuraj operacioj              |
| `pub`     | `pub`   | Publika videbleco                 |

## Datumtipoj

| Esperanto  | Rust     | Priskribo                        |
|------------|----------|----------------------------------|
| `ent8`     | `i8`     | Entjero kun signo (8-bit)        |
| `ent16`    | `i16`    | Entjero kun signo (16-bit)       |
| `ent32`    | `i32`    | Entjero kun signo (32-bit)       |
| `ent64`    | `i64`    | Entjero kun signo (64-bit)       |
| `ent128`   | `i128`   | Entjero kun signo (128-bit)      |
| `entg`     | `isize`  | Entjero kun signo (platforma)    |
| `nat8`     | `u8`     | Sensigna entjero (8-bit)         |
| `nat16`    | `u16`    | Sensigna entjero (16-bit)        |
| `nat32`    | `u32`    | Sensigna entjero (32-bit)        |
| `nat64`    | `u64`    | Sensigna entjero (64-bit)        |
| `nat128`   | `u128`   | Sensigna entjero (128-bit)       |
| `natg`     | `usize`  | Sensigna entjero (platforma)     |
| `glit32`   | `f32`    | Glitkoma nombro (32-bit)         |
| `glit64`   | `f64`    | Glitkoma nombro (64-bit)         |
| `bulea`    | `bool`   | Logika valoro                    |
| `kar`      | `char`   | Unikoda signo                    |
| `ĉen`      | `str`    | Signoĉeno (ne dinamika)          |
| `Teksto`   | `String` | Dinamika signoĉeno               |
| `Vektoro`  | `Vec`    | Dinamika tabelo                  |

## Buleaj valoroj

| Esperanto  | Rust    |
|------------|---------|
| `vera`     | `true`  |
| `malvera`  | `false` |

## Arkitekturo

La projekto konsistas el kvar kestoj (crates):

```
├── Cargo.toml          # Laborspaca difino
├── crates/
│   ├── resp-ast/       # AST-difinoj por la lingvo
│   ├── resp-parser/    # Sintaksa analizilo (pest PEG)
│   ├── resp-transpiler/# Rust-kodgeneratoro (syn/quote)
│   └── resp-cli/       # Komandlinia interfaco
├── examples/           # Ekzemplaj .resp dosieroj
└── tests/              # Integrigaj testoj
```

## Disvolvado

```bash
# Konstrui
cargo build

# Testi
cargo test

# Formati
cargo fmt

# Lint
cargo clippy
```

## Permesilo

MIT
