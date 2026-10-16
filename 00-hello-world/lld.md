<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

**Table of Contents** _generated with [DocToc](https://github.com/thlorenz/doctoc)_

- [Low-Level Design (LLD) — Standard Terminal Hello World](#low-level-design-lld--standard-terminal-hello-world)
  - [1. Source layout](#1-source-layout)
  - [2. Data types](#2-data-types)
    - [2.1 `cli::ParsedArgs`, `cli::OptionValue`, `cli::Invocation`](#21-cliparsedargs-clioptionvalue-cliinvocation)
    - [2.2 `validate::Config`](#22-validateconfig)
    - [2.3 Error type `diagnostics::UsageError`](#23-error-type-diagnosticsusageerror)
  - [3. Module and function specifications](#3-module-and-function-specifications)
    - [3.1 `arguments::from_os_args` / `to_utf8_bytes`](#31-argumentsfrom_os_args--to_utf8_bytes)
    - [3.2 `cli::parse`](#32-cliparse)
    - [3.3 `cli::parse_short_cluster` (private)](#33-cliparse_short_cluster-private)
    - [3.4 `validate::validate`, `validate_name`, `validate_count`](#34-validatevalidate-validate_name-validate_count)
    - [3.5 `greeting::greeting_line`, `greeting_text`](#35-greetinggreeting_line-greeting_text)
    - [3.6 `diagnostics::UsageError::message`, `report`](#36-diagnosticsusageerrormessage-report)
    - [3.7 `output::write_output`](#37-outputwrite_output)
    - [3.8 `run` (`src/lib.rs`)](#38-run-srclibrs)
  - [4. Algorithms](#4-algorithms)
  - [5. Platform-specific details](#5-platform-specific-details)
  - [6. Resource management](#6-resource-management)
  - [7. Sequence of one invocation](#7-sequence-of-one-invocation)
  - [8. Design-to-code checklist](#8-design-to-code-checklist)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Low-Level Design (LLD) — Standard Terminal Hello World

| Field                   | Value                                                 |
| :---------------------- | :---------------------------------------------------- |
| Project                 | `00-hello-world`                                      |
| Tools                   | dp-hello                                              |
| Implementation language | rs (Rust 1.99, edition 2024)                          |
| Author                  | Brandon Perkins                                       |
| Version / date          | 1.0 — 2026-10-13                                      |
| Status                  | Approved                                              |
| Inputs                  | `hld.md`, `specs/SRS.md`, `shared/CLI-CONVENTIONS.md` |

## 1. Source layout

One cargo package (`Cargo.toml`, `Cargo.lock`, no dependencies): library crate `dp_hello` plus
binary target `dp-hello`. Unit tests live in a `#[cfg(test)] mod tests` at the end of each file.

```text
00-hello-world/
├── Cargo.toml, Cargo.lock
└── src/
    ├── main.rs         # entry point: args_os → run → ExitCode
    ├── lib.rs          # run(): pipeline and exit statuses; module map
    ├── arguments.rs    # OS arguments → UTF-8 bytes (platform boundary)
    ├── cli.rs          # option table and parser
    ├── validate.rs     # Config, name and count validation
    ├── greeting.rs     # greeting_line / greeting_text, HELP_TEXT, VERSION_LINE
    ├── diagnostics.rs  # UsageError, message(), report()
    └── output.rs       # write_output(), WriteOutcome
```

## 2. Data types

### 2.1 `cli::ParsedArgs`, `cli::OptionValue`, `cli::Invocation`

| Type / field          | Type                                        | Default | Constraint / invariant                         | SRS            |
| :-------------------- | :------------------------------------------ | :------ | :--------------------------------------------- | :------------- |
| `OptionValue.value`   | `Vec<u8>`                                   | —       | Bytes exactly as given                         | FR-02, CONV-05 |
| `OptionValue.option`  | `&'static str`                              | —       | `-n`, `--name`, `-c`, or `--count`, as written | FR-03, FR-05   |
| `ParsedArgs.name`     | `Option<OptionValue>`                       | `None`  | Last occurrence                                | FR-06          |
| `ParsedArgs.count`    | `Option<OptionValue>`                       | `None`  | Last occurrence                                | FR-06          |
| `ParsedArgs.operands` | `Vec<Vec<u8>>`                              | empty   | In command-line order                          | FR-09          |
| `Invocation`          | enum `Help`, `Version`, `Greet(ParsedArgs)` | —       | `Help`/`Version` only when reached first       | FR-07, FR-08   |

### 2.2 `validate::Config`

| Field   | Type      | Default    | Constraint / invariant | SRS           |
| :------ | :-------- | :--------- | :--------------------- | :------------ |
| `name`  | `Vec<u8>` | `b"World"` | Non-empty              | FR-01 … FR-03 |
| `count` | `u16`     | 1          | 1 ≤ count ≤ 1000       | FR-04, FR-05  |

### 2.3 Error type `diagnostics::UsageError`

Every variant produces `dp-hello: <message>` LF `Try 'dp-hello --help' for more information.` LF
on stderr and exit status 2.

| Variant                             | Message                                     | Example trigger    |
| :---------------------------------- | :------------------------------------------ | :----------------- |
| `InvalidOption(letter)`             | `invalid option -- '<letter>'`              | `-x`               |
| `UnrecognizedOption(token)`         | `unrecognized option '<token>'`             | `--nme`, `--nam=x` |
| `ShortOptionRequiresArgument(c)`    | `option requires an argument -- '<c>'`      | `-n` last          |
| `LongOptionRequiresArgument(opt)`   | `option '<opt>' requires an argument`       | `--count` last     |
| `ArgumentNotAllowed(opt)`           | `option '<opt>' doesn't allow an argument`  | `--help=x`         |
| `InvalidArgument { value, option }` | `invalid argument '<value>' for '<option>'` | `-c 0`, `--name=`  |
| `ExtraOperand(operand)`             | `extra operand '<operand>'`                 | `extra`, `-- -n`   |

## 3. Module and function specifications

### 3.1 `arguments::from_os_args` / `to_utf8_bytes`

- **Signature:** `fn from_os_args(args: impl IntoIterator<Item = OsString>) -> Vec<Vec<u8>>`;
  `fn to_utf8_bytes(arg: OsString) -> Vec<u8>` (two `cfg` variants).
- **Responsibility:** produce the UTF-8 bytes of each argument.
- **Postconditions:** Unix: the exact argument bytes. Windows: UTF-8 of the UTF-16 argument,
  with unpaired surrogates replaced by U+FFFD. Never panics.
- **Satisfies:** CONV-05, CONV-24. **Unit tests:** `arguments_keep_utf8_text`,
  `arguments_keep_invalid_utf8_bytes_on_unix`, `arguments_replace_unpaired_surrogates_on_windows`.

### 3.2 `cli::parse`

- **Signature:** `pub fn parse(args: &[Vec<u8>]) -> Result<Invocation, UsageError>`
- **Responsibility:** recognize option syntax, left to right.
- **Errors:** `InvalidOption`, `UnrecognizedOption`, `ShortOptionRequiresArgument`,
  `LongOptionRequiresArgument`, `ArgumentNotAllowed`.
- **Algorithm:** for each token:
  1. `--`: append all remaining tokens to `operands`.
  2. `--body`: split `body` at the first `=`; look the name up in `OPTIONS` (exact match, so
     no abbreviations). Unknown → `UnrecognizedOption(token)`. `--help`/`--version`: with `=`
     → `ArgumentNotAllowed`, otherwise return `Help`/`Version` immediately. `--name`/`--count`:
     the value is the inline part, else the next token, else `LongOptionRequiresArgument`.
  3. `-` followed by at least one byte: `parse_short_cluster` (3.3).
  4. Anything else (including `-`): push to `operands`.
  5. At the end, return `Greet(parsed)`.
- **Complexity:** O(total argument bytes) time and memory.
- **Satisfies:** FR-02, FR-06, FR-07, FR-08, FR-10; CONV-07 … CONV-12, CONV-22.
- **Unit tests:** every `parse_*` test in `src/cli.rs`.

### 3.3 `cli::parse_short_cluster` (private)

- Splits off one character at a time with `split_first_char` (one UTF-8 scalar, or one byte of
  an invalid sequence, via `<[u8]>::utf8_chunks`). Unknown → `InvalidOption(char bytes)`. A
  value-taking option consumes the rest of the token (`-c2`), or the next token when the rest
  is empty, or fails with `ShortOptionRequiresArgument`. **Satisfies:** CONV-07.

### 3.4 `validate::validate`, `validate_name`, `validate_count`

- **Signature:** `pub fn validate(parsed: ParsedArgs) -> Result<Config, UsageError>`
- **Algorithm:** (1) first operand → `ExtraOperand`; (2) name: absent → `World`, empty →
  `InvalidArgument`; (3) count: absent → 1; otherwise every byte must be `0`–`9` and the value
  is accumulated in a `u16`, failing as soon as it exceeds 1000 (overflow-free for any
  length), and must be ≥ 1.
- **Satisfies:** FR-03, FR-04, FR-05, FR-06, FR-09.
- **Unit tests:** every `validate_*` test in `src/validate.rs`.

### 3.5 `greeting::greeting_line`, `greeting_text`

- **Signatures:** `pub fn greeting_line(name: &[u8]) -> Vec<u8>`;
  `pub fn greeting_text(name: &[u8], count: u16) -> Vec<u8>`.
- **Responsibility:** pure construction of `Hello, <name>!\n`, repeated `count` times.
- **Complexity:** O(count × len(name)).
- **Satisfies:** FR-01, FR-02, FR-04, NFR-05. **Unit tests:** `greeting_line_default_name`,
  `greeting_line_keeps_name_verbatim`, `greeting_text_repeats_lines`,
  `help_and_version_end_in_lf_only` (for `HELP_TEXT`, `VERSION_LINE`).

### 3.6 `diagnostics::UsageError::message`, `report`

- **Responsibility:** the exact CONV-13 text (bytes, values inserted without escaping) and the
  full two-line stderr report (CONV-14).
- **Unit tests:** `diagnostics_match_conv13`, `diagnostics_report_adds_prefix_and_hint`,
  `diagnostics_echo_values_without_escaping`.

### 3.7 `output::write_output`

- **Signature:** `pub fn write_output(stream: &mut impl Write, bytes: &[u8]) -> WriteOutcome`
- **Algorithm:** `write_all` then `flush`; `Ok` → `Written`; `ErrorKind::BrokenPipe` →
  `ReaderGone`; anything else → `Failed`.
- **Satisfies:** CONV-03, CONV-25. **Unit tests:** `output_writes_bytes_unchanged`,
  `output_broken_pipe_is_quiet`, `output_other_errors_fail`.

### 3.8 `run` (`src/lib.rs`)

- **Signature:** `pub fn run(arguments: &[Vec<u8>], stdout: &mut impl Write, stderr: &mut impl Write) -> u8`
- **Algorithm:** `parse` → help/version text, or `validate` → `greeting_text`; write to stdout
  and return 0 (`Written`, `ReaderGone`) or 1 (`Failed`); on `UsageError`, write `report()` to
  stderr and return 2.
- **Satisfies:** FR-07, FR-08, FR-11, §3.7. **Unit tests:** every `run_*` test in `src/lib.rs`.

## 4. Algorithms

The only non-trivial algorithms are the option scan (3.2, 3.3) and count parsing (3.4). The scan
is a single pass with an iterator, so a value-taking option can consume the following token.
Count parsing accumulates digit by digit and stops at the first value above 1000, which avoids
both integer overflow and arbitrary-precision arithmetic.

## 5. Platform-specific details

| Concern             | Linux                                            | macOS      | Windows                                               | Abstraction                              |
| :------------------ | :----------------------------------------------- | :--------- | :---------------------------------------------------- | :--------------------------------------- |
| Argument encoding   | bytes                                            | bytes      | UTF-16 → UTF-8                                        | `arguments::to_utf8_bytes` (`cfg(unix)`) |
| Newline translation | none                                             | none       | none (Rust never translates)                          | —                                        |
| Console output      | bytes                                            | bytes      | console: UTF-8 → UTF-16 by std; redirected: raw bytes | `std::io::Stdout`                        |
| SIGPIPE             | ignored by the Rust runtime → `BrokenPipe` error | same       | `ERROR_NO_DATA` → `BrokenPipe`                        | `output::write_output`                   |
| Executable name     | `dp-hello`                                       | `dp-hello` | `dp-hello.exe`                                        | Makefile `$(EXE)`                        |

## 6. Resource management

No files, sockets, or child processes. stdout and stderr are locked once in `main` and released
when it returns. The only buffers are the argument vectors and the output block (≤ 1000 lines).

## 7. Sequence of one invocation

Public test 06, `dp-hello -c2 -n Ken`: `main` converts the arguments to `[b"-c2", b"-n", b"Ken"]`.
`cli::parse` sees `-c2` → `parse_short_cluster` → `c` takes the rest `2` → `count = ("2", "-c")`;
`-n` → empty rest → next token `Ken` → `name = ("Ken", "-n")`; end → `Greet`. `validate` →
`Config { name: "Ken", count: 2 }`. `greeting_text` → `Hello, Ken!\nHello, Ken!\n`.
`write_output(stdout)` → `Written` → exit 0.

## 8. Design-to-code checklist

| HLD component | LLD section | Source file(s)                    | Unit-test file(s)    |
| :------------ | :---------- | :-------------------------------- | :------------------- |
| `arguments`   | §3.1        | `src/arguments.rs`, `src/main.rs` | `src/arguments.rs`   |
| `cli`         | §3.2, §3.3  | `src/cli.rs`                      | `src/cli.rs`         |
| `validate`    | §3.4        | `src/validate.rs`                 | `src/validate.rs`    |
| `greeting`    | §3.5        | `src/greeting.rs`                 | `src/greeting.rs`    |
| `diagnostics` | §3.6        | `src/diagnostics.rs`              | `src/diagnostics.rs` |
| `output`      | §3.7        | `src/output.rs`                   | `src/output.rs`      |
| `run`         | §3.8        | `src/lib.rs`                      | `src/lib.rs`         |
