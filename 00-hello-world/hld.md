# High-Level Design (HLD) — Standard Terminal Hello World

| Field | Value |
| :-- | :-- |
| Project | `00-hello-world` |
| Tools | dp-hello |
| Implementation language | rs (Rust 1.99, edition 2024) |
| Author | Brandon Perkins |
| Version / date | 1.0 — 2026-10-13 |
| Status | Approved |
| Inputs | `specs/URD.md`, `specs/SRS.md`, `specs/user-manual.md`, `shared/CLI-CONVENTIONS.md` |

## 1. Introduction

### 1.1 Scope
`dp-hello` writes `Hello, ` + NAME + `!` + LF to standard output COUNT times (NAME defaults to
`World`, COUNT to 1, range 1–1000), prints help and version text, and rejects every usage error
with the exact CONV-13/14 diagnostics and exit status 2. As the SRS (§1.2) states, it reads no
input, takes no operands, touches no files, and reads no environment variables.

### 1.2 Definitions and acronyms
| Term | Meaning |
| :-- | :-- |
| Token | One command-line argument, held as its UTF-8 bytes (`Vec<u8>`) |
| Spelling | The option as the user wrote it (`-n` or `--name`), echoed in diagnostics |
| Invocation | The parser's result: help, version, or a greeting request to validate |
| EPIPE | The reader of standard output has gone away (broken pipe) |

### 1.3 References
- SRS §3 (interfaces), §4 (FR-01 … FR-11), §5 (NFR-01 … NFR-07), §6 (diagnostics).
- CLI-CONV §2 (CONV-03, CONV-05), §3 (CONV-07 … CONV-12), §4.1 (CONV-13 … CONV-15), §8.
- User manual, "Help output" (normative help text).

## 2. Requirements summary

| Area | SRS requirements | Architectural impact |
| :-- | :-- | :-- |
| Option parsing | FR-02, FR-06, FR-07, FR-08, FR-10; CONV-07 … CONV-12 | A table-driven, hand-written parser (`cli`) that stops at `--help`/`--version` |
| Validation | FR-03, FR-04, FR-05, FR-09 | A separate `validate` step that checks only the winning values |
| Greeting construction | FR-01, FR-02, FR-04; NFR-05 | Pure functions over bytes (`greeting`) |
| Diagnostics | FR-11; CONV-13 … CONV-15, CONV-24 | One error enum whose rendering is the single source of message text |
| Output and portability | NFR-01; CONV-03, CONV-05, CONV-25 | Byte-oriented arguments and output; a platform boundary for arguments |
| Build | NFR-06 | A cargo package with Makefile hooks |

## 3. Architectural overview

### 3.1 Context diagram

```text
  argv (OS) ──▶ ┌──────────┐ ──▶ stdout (greeting / help / version)
                │ dp-hello │ ──▶ stderr (usage diagnostics)
                └──────────┘ ──▶ exit status 0 or 2
  No stdin, files, environment variables, subprocesses, or network.
```

### 3.2 Component (module) decomposition

| Component | Responsibility | Depends on |
| :-- | :-- | :-- |
| `main` (`src/main.rs`) | Collect `args_os`, lock the streams, return the exit code | `arguments`, `run` |
| `arguments` | Convert OS arguments to UTF-8 bytes (the only platform-specific code) | std |
| `cli` | Turn tokens into an `Invocation` or a `UsageError` (syntax only) | `diagnostics` |
| `validate` | Turn `ParsedArgs` into a `Config` (operands, name, count) | `cli`, `diagnostics` |
| `greeting` | Build greeting lines; hold the help and version text (pure) | — |
| `diagnostics` | `UsageError` and its exact message and report | — |
| `output` | Write a byte block to a stream; classify EPIPE | std |
| `run` (`src/lib.rs`) | Wire the pipeline and map outcomes to exit statuses | all of the above |

### 3.3 Data flow
`main` → `arguments::from_os_args` → `run`: `cli::parse` → (`Help`/`Version` → fixed text) or
(`Greet` → `validate::validate` → `greeting::greeting_text`) → `output::write_output(stdout)` →
exit 0. Any `UsageError` short-circuits to `output::write_output(stderr, error.report())` →
exit 2, so stdout stays empty (FR-11).

## 4. Key design decisions

| # | Decision | Alternatives considered | Rationale |
| :-- | :-- | :-- | :-- |
| D1 | Arguments are byte vectors (`Vec<u8>`) throughout | `String` via `args()` (panics on invalid UTF-8); lossy `String` | Never panics (CONV-24) and echoes POSIX argument bytes exactly (CONV-05) |
| D2 | Build the whole output in memory, then one `write_all` | Write line by line | At most 1000 short lines; errors can't interleave with output; one syscall |
| D3 | Errors are values (`Result<_, UsageError>`), rendered in one place | Printing where detected; `process::exit` deep in the code | One mapping to messages and exit status; testable without a process |
| D4 | Parse first, validate after the scan | Validate while scanning | FR-06: only the winning value is validated; CONV-12: `--help` after an invalid value still wins |
| D5 | Hand-written, table-driven parser | clap, getopts | Argument-parsing libraries are out of bounds; the table keeps short/long handling DRY |
| D6 | `run` takes generic `Write` streams | Use the global stdout/stderr | End-to-end unit tests with in-memory buffers |

## 5. Cross-platform strategy
- **Arguments (CONV-05):** `arguments` is the only module with `cfg` code. On Unix the bytes are
  taken verbatim (`OsStringExt::into_vec`). On Windows, `std::env::args_os` decodes the UTF-16
  command line, which is converted to UTF-8; unpaired surrogates become U+FFFD.
- **Line endings (CONV-03):** every generated line ends in `\n`, and Rust's `Stdout`/`Stderr`
  never translate newlines, so output is effectively binary-mode on Windows too.
- **Paths (CONV-19 … CONV-21):** not applicable; the tool touches no files.

## 6. Error handling and diagnostics
Parsing and validation return `Result<_, UsageError>`. `UsageError::message` holds the only copy
of the CONV-13 wording, and `UsageError::report` adds the `dp-hello: ` prefix and the CONV-14
hint. `run` is the only place that maps outcomes to exit statuses: success, help, and version
→ 0; usage error → 2. A broken pipe on stdout ends the tool quietly with status 0 (CONV-25, status
unspecified). Any other write failure, which the SRS does not cover (for example a full disk),
exits 1 without a diagnostic. No code path calls `unwrap`, `expect`, or indexing that can fail on
user input, so no argument list can cause a panic (CONV-24).

## 7. Non-functional design

| Quality attribute | SRS reference | How the design achieves it |
| :-- | :-- | :-- |
| Performance / memory | NFR-02 | Native release build (LTO); at most ~1000 × (len(NAME) + 9) bytes buffered; measured < 10 ms and ~2 MiB RSS for `-c 1000` |
| Portability | NFR-01 | Byte-oriented I/O, LF-only constants, the `arguments` platform boundary |
| Robustness / security | NFR-03, NFR-07 | Reads only argv; overflow-free count parsing; no files, env vars, or subprocesses |
| Maintainability | NFR-04, NFR-05 | One module per responsibility; pure `greeting` functions; unit tests next to the code |

## 8. Risks and open issues

| Risk / issue | Likelihood | Impact | Mitigation |
| :-- | :-- | :-- | :-- |
| Windows console rejects non-UTF-8 output | L | L | Windows arguments are always valid UTF-8 after conversion (D1) |
| Operand errors versus `--help` in scan order | M | M | CONV-10/12: syntax errors and `--help` are handled in scan order; operands are checked after the scan, so `extra --help` prints help |
| Toolchain drift between the container and native CI | L | M | `make toolchain` checks Rust 1.99, clippy, and rustfmt everywhere |

## 9. Approval

| Role | Name | Date | Decision |
| :-- | :-- | :-- | :-- |
| Author | Brandon Perkins | 2026-10-13 | Submitted |
| Reviewer | CI (container; native Linux, macOS, Windows) | 2026-10-13 | Approved when every check passes |
