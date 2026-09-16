# AGENTS.md — guia de desenvolvimento do RustCalc

Calculadora gráfica em Rust. GUI em **egui/eframe 0.36**; a engine de
cálculo (`src/eval.rs`) é **stdlib pura** (não usa `egui`, não usa crates
externos) e é o único lugar com lógica não-trivial.

## Comandos

```sh
cargo build                        # debug
cargo run --release                # executa
cargo test                         # 13 testes de integração (tests/eval.rs)
cargo clippy --all-targets -- -D warnings
cargo fmt --check                  # rode `cargo fmt` antes de concluir
```

## Estrutura

- `src/main.rs` — UI: `CalcApp` implementa `eframe::App::ui(&mut Ui, …)`.
  Modo básico = keypad 5 linhas; científico = keypad + painel de funções.
  Temas, fontes e strings são definidos aqui.
- `src/eval.rs` — `evaluate(&str, Options)`, `format_value(f64)`,
  `Options { angle: Angle, ans: f64 }`. Normaliza símbolos tipográficos.
- `src/lib.rs` — `pub mod eval` (tests de integração usam a lib).
- `assets/fonts/` — DejaVu; carregados com `include_bytes!` em `setup_fonts()`.
- `docs/screenshot-basic.png` — captura da interface (referenciada no README).

## Convenções

- **UI, mensagens de erro e documentação em PT-BR** (dígito decimal `.`);
  identificadores de código em inglês.
- Engine pura: `eval.rs` não pode depender de `egui` nem ter estado global;
  todo estado relevante passa por `Options`.
- Não adicione dependências novas sem necessidade — `src/eval.rs` e
  `tests/eval.rs` devem permanecer buildáveis offline (sem rede).
- Lógica não-trivial na engine exige cobertura em `tests/eval.rs` e o projeto
  deve passar `cargo clippy --all-targets -- -D warnings` e `cargo fmt`.

## Ambiente deste repositório (Windows)

- Toolchain: `stable-x86_64-pc-windows-gnu` (sem MSVC/link.exe na máquina).
- Linker MinGW-W64 WinLibs precisa estar no PATH **toda vez** (não persiste
  entre shells):

```powershell
$env:PATH = "C:\Users\fulano\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;$env:PATH"
cargo build
```

- `static.rust-lang.org` é bloqueado nesta rede; se o rustup precisar baixar
  uma toolchain, use o mirror:

```powershell
$env:RUSTUP_DIST_SERVER = "https://mirrors.ustc.edu.cn/rust-static"
rustup toolchain install stable-x86_64-pc-windows-gnu
```

- Como a build roda em `wasm-pack`? Não roda: é nativa só de desktop.