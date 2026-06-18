# Kontribuado al Resp

Dankon pro via intereso kontribui al Resp! Jen kelkaj gvidlinioj por helpi vin.

## Raporti Problemajn

- Uzu la [GitHub Issues](https://github.com/SrDicov/Resp/issues) por raporti cimojn
- Inkluzivu ekzemplan `.resp` dosieron kaj la atenditan rezulton
- Se eble, inkluzivu la erarmesaĝon

## Kontribui Kodon

1. Forku la deponejon
2. Kreu novan branĉon: `git switch -c mia-funkcio`
3. Faru viajn ŝanĝojn
4. Certigu, ke ĉio konstruiĝas: `cargo build`
5. Plenumi testojn: `cargo test`
6. Sendu Pull Request

## Kodaj Normoj

- Sekvu la ekzistantan kodian stilon
- Uzu `cargo fmt` antaŭ ol sendi
- Ne inkluzivu komentariojn en la kodo (krom dokumentado)
- Skribu testojn por novaj funkcioj

## Strukturo de la Projekto

- `resp-ast` - AST-tipoj (neniu ekstera dependeco)
- `resp-parser` - Analizilo uzanta pest PEG-gramatikon
- `resp-transpiler` - Rust-kodgeneratoro uzanta syn/quote
- `resp-cli` - CLI-ilo uzanta clap

## Aldoni Novan Ŝlosilvorton

1. Aldoni la regulon en `crates/resp-parser/grammar/esperanto.pest`
2. Aldoni la AST-enumon en `crates/resp-ast/src/lib.rs`
3. Aldoni la analizan logikon en `crates/resp-parser/src/lib.rs`
4. Aldoni la transpilan logikon en `crates/resp-transpiler/src/lib.rs`
5. Aldoni la ŝlosilvorton al la mapo en `README.md`
6. Verki testojn
