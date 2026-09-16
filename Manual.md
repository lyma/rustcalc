# Manual do RustCalc

Uma calculadora em Rust com dois modos: **Básica** e **Científica**.
Use o mouse ou o teclado físico. O `Enter` no campo de expressão equivale a `=`.

## Onde baixar

- **Windows (recomendado):** baixe o `rustcalc.exe` pronto em
  https://github.com/lyma/rustcalc/releases — não precisa instalar nada.
- **Compilar do código:** [Rust](https://rustup.rs) instalado, então
  `cargo run --release` na raiz do projeto.

---

## Tela

- **Campo de expressão** (direita, monoespaçado): mostra o que você digita e,
  depois de `=`, o resultado.
- **Linha acima**: `ans = …` mostra o último resultado calculado.
- **Indicador M**: aparece quando há valor guardado na memória.

## Modo Básico

| Tecla | O que faz |
|---|---|
| `0`–`9`, `.` | Dígitos e separador decimal |
| `+` `−` `×` `÷` | Operações |
| `=` | Calcula |
| `±` | Troca o sinal do número exibido |
| `%` | Porcentagem (ver abaixo) |
| `AC` | Limpa tudo |
| `MC` `MR` `M+` `M−` | Memória |

Comportamentos:

- Depois de um `=`, digitar outro número **começa uma conta nova** pelo número
  digitado; digitar um operador **continua** a partir do resultado.
- Digitar um operador logo após outro **troca** o último operador.
- `%` pós-fixo divide por 100: `50%` → `0,5`. Entre dois números é resto:
  `8%3` → `2`. Em `250+10%` o resultado é `250,1`.

## Modo Científico

Além do teclado básico, um painel à direita adiciona:

- `sin()` `cos()` `tan()` `asin()` `acos()` `atan()` — trigonometria.
- `ln()` `log()` `log2()` — logaritmos natural, decimal e base 2.
- `√()` `∛()` `exp()` — riz raiz quadrada, cúbica e exponencial.
- `π` `e` — constantes.
- `x!` `x²` `x³` `x^y` `10^x` `|x|` `ans` — demais operações, módulo e
  último resultado.
- `RAD`/`DEG` — alterna radianos/graus (funciona para `sin`/`cos`/`tan` e seus
  inversos).
- `Del` — apaga o último caractere.

> Dica: `√(` e `∛(` inserem `sqrt(` e `cbrt(`; o parêntese final é fechado
> automaticamente ao calcular. Você pode digitar `√9` e pressionar `=`.

## Sintaxe aceita

- Operadores: `+` `-` `*` `/` `%` `^` `!`.
- Parênteses `(` `)` e separador decimal `.` (vírgulas entre argumentos são
  aceitas como separador de lista de funções).
- Multiplicação implícita: `2π`, `2(3+4)`, `(1+2)(3)`.
- Notação científica: `1e3` (= 1000), `1.5e-3`.
- Caracteres tipográficos: `π × ÷ − ² ³ √` são normalizados
  (`²` = `^2`, `√9` = raiz de 9).
- Constantes: `pi`, `e`, `tau`, `ans` (último resultado).

Precedência (menor → maior):

1. `+` `-`
2. `*` `/` `%` (resto)
3. sinal unário (`-2^2` = `-(2^2)` = `-4`)
4. `^` associa à direita (`2^3^2` = `512`)
5. pós-fixo `!` (fatorial) e `%` (porcentagem)

### Funções

Um argumento: `sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh ln
log log2 sqrt cbrt abs floor ceil round sign exp`.

Dois argumentos: `pow(min,max)` → `pow(a,b)`, `min(a,b)`, `max(a,b)`,
`hypot(a,b)`.

## Erros

| Mensagem | Causa comum |
|---|---|
| `Divisão por zero` | Divisão ou resto por zero |
| `Domínio inválido: …` | `sqrt(-1)`, `ln(0)`, `asin(2)` etc. |
| `Fatorial inválido para …` | Fatorial de número não inteiro |
| `Resultado fora do intervalo suportado` | `2^2000`, `171!` |
| `Token inesperado: …` | Parêntese extra, `sin(1,2)` mal usada etc. |
| `Função ou constante desconhecida: …` | Nome digitado não existe |
| `Valor esperado` | Expressão incompleta (ex.: `2+`) |

Dica: `AC` (ou `Del`) limpa o estado de erro.