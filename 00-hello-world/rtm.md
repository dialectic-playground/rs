<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

**Table of Contents** _generated with [DocToc](https://github.com/thlorenz/doctoc)_

- [Requirements Traceability Matrix (RTM) — Standard Terminal Hello World](#requirements-traceability-matrix-rtm--standard-terminal-hello-world)
  - [1. User requirements to system requirements](#1-user-requirements-to-system-requirements)
  - [2. Forward traceability](#2-forward-traceability)
  - [3. Backward traceability (test → requirement)](#3-backward-traceability-test-%E2%86%92-requirement)
  - [4. Coverage summary](#4-coverage-summary)
  - [5. Gaps and justifications](#5-gaps-and-justifications)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Requirements Traceability Matrix (RTM) — Standard Terminal Hello World

| Field          | Value            |
| :------------- | :--------------- |
| Project        | `00-hello-world` |
| Tools          | dp-hello         |
| Author         | Brandon Perkins  |
| Version / date | 1.0 — 2026-10-13 |

Code paths are relative to `00-hello-world/`. Unit tests run with `cargo test` (`make test`);
public black-box tests are `P01` … `P10`.

## 1. User requirements to system requirements

| User requirement (URD) | Derived system requirements (SRS) |
| :--------------------- | :-------------------------------- |
| UR-01                  | FR-01                             |
| UR-02                  | FR-02                             |
| UR-03                  | FR-04                             |
| UR-04                  | FR-07                             |
| UR-05                  | FR-08                             |
| UR-06                  | FR-03, FR-05, FR-09, FR-10, FR-11 |
| UR-07                  | FR-01, SRS §3.7                   |
| UR-08                  | NFR-01, CONV-03                   |
| UR-09                  | FR-02, CONV-05                    |
| UR-10                  | FR-06, FR-10, CONV-07 … CONV-15   |
| UR-11                  | NFR-02                            |
| UR-12                  | NFR-06                            |

## 2. Forward traceability

| Req. ID       | Summary                               | Design (HLD/LLD)         | Code (file::function)                                                                 | Verification (unit / black-box)                                                                                                                          | Status   |
| :------------ | :------------------------------------ | :----------------------- | :------------------------------------------------------------------------------------ | :------------------------------------------------------------------------------------------------------------------------------------------------------- | :------- |
| UR-01 … UR-12 | User needs                            | HLD §2                   | via the FR/NFR rows below                                                             | via the FR/NFR rows below                                                                                                                                | Verified |
| FR-01         | Default greeting                      | HLD §3.3, LLD §3.4, §3.5 | `src/validate.rs::validate`, `src/greeting.rs::greeting_line`                         | `validate_defaults`, `greeting_line_default_name`, `run_default_greeting`, P01                                                                           | Verified |
| FR-02         | Named greeting, verbatim              | HLD D1, LLD §3.2, §3.5   | `src/cli.rs::parse`, `src/greeting.rs::greeting_line`                                 | `parse_name_spellings`, `parse_name_may_start_with_dash`, `greeting_line_keeps_name_verbatim`, `run_repeated_named_greeting`, P02–P04                    | Verified |
| FR-03         | Empty name rejected                   | LLD §3.4                 | `src/validate.rs::validate_name`                                                      | `validate_empty_name`, `parse_empty_attached_values`                                                                                                     | Verified |
| FR-04         | Repetition 1–1000                     | LLD §3.4, §3.5           | `src/validate.rs::validate_count`, `src/greeting.rs::greeting_text`                   | `validate_count_boundaries`, `validate_count_leading_zeros`, `greeting_text_repeats_lines`, P05, P06                                                     | Verified |
| FR-05         | Invalid count rejected                | LLD §3.4                 | `src/validate.rs::validate_count`                                                     | `validate_count_boundaries`, `validate_count_non_digits`, `validate_count_reports_spelling`, `run_usage_error_writes_only_stderr`                        | Verified |
| FR-06         | Last occurrence wins                  | HLD D4, LLD §3.2         | `src/cli.rs::store`                                                                   | `parse_last_occurrence_wins`, `validate_only_winning_values`                                                                                             | Verified |
| FR-07         | Help text                             | LLD §3.2, §3.5           | `src/cli.rs::parse`, `src/greeting.rs::HELP_TEXT`                                     | `parse_help_and_version_stop_the_scan`, `run_help_and_version_exit_zero`, P07                                                                            | Verified |
| FR-08         | Version line                          | LLD §3.2, §3.5           | `src/greeting.rs::VERSION_LINE`                                                       | `run_help_and_version_exit_zero`, P08                                                                                                                    | Verified |
| FR-09         | Operands rejected                     | LLD §3.4                 | `src/validate.rs::validate`                                                           | `parse_collects_operands`, `validate_operands_rejected_first`, P10                                                                                       | Verified |
| FR-10         | Unknown / malformed options           | LLD §2.3, §3.2, §3.3     | `src/cli.rs::parse`, `src/cli.rs::parse_short_cluster`                                | `parse_unknown_short_option`, `parse_unrecognized_long_options`, `parse_missing_arguments`, `parse_help_and_version_take_no_argument`, P09               | Verified |
| FR-11         | No stdout on error                    | HLD §3.3, LLD §3.8       | `src/lib.rs::run`                                                                     | `run_usage_error_writes_only_stderr`, P09, P10                                                                                                           | Verified |
| NFR-01        | Byte-identical, LF only               | HLD §5                   | `src/greeting.rs`, `src/output.rs::write_output`                                      | `help_and_version_end_in_lf_only`, `output_writes_bytes_unchanged`, P01–P10 on Linux, macOS, Windows CI                                                  | Verified |
| NFR-02        | < 100 ms, < 32 MiB                    | HLD §7                   | release build (`Cargo.toml` profile)                                                  | Measurement: `-c 1000` in < 10 ms, ~2 MiB RSS (MTP §4.4)                                                                                                 | Verified |
| NFR-03        | No crash / panic                      | HLD §6, D1               | `src/arguments.rs`, `src/cli.rs::split_first_char`, `src/validate.rs::validate_count` | `parse_unknown_short_option_with_invalid_utf8`, `arguments_keep_invalid_utf8_bytes_on_unix`, `validate_count_boundaries`, `run_output_failure_is_silent` | Verified |
| NFR-04        | Separate units                        | HLD §3.2                 | `src/{cli,validate,greeting,output}.rs`                                               | Design review (this RTM, LLD §8)                                                                                                                         | Verified |
| NFR-05        | Pure, tested greeting                 | LLD §3.5                 | `src/greeting.rs::greeting_line`, `greeting_text`                                     | `greeting_line_*`, `greeting_text_repeats_lines`                                                                                                         | Verified |
| NFR-06        | Every make target works               | —                        | `Makefile` hooks                                                                      | CI `make check roundtrip` (container + 3 OSes), `make format`/`clean`/`distclean`                                                                        | Verified |
| NFR-07        | No files, env, subprocesses           | HLD §7                   | `src/main.rs::main` (reads only `args_os`)                                            | Inspection                                                                                                                                               | Verified |
| CONV-01       | Canonical name                        | LLD §3.6                 | `src/lib.rs::TOOL`                                                                    | `diagnostics_report_adds_prefix_and_hint`, P07–P10                                                                                                       | Verified |
| CONV-02       | Version format                        | LLD §3.5                 | `src/greeting.rs::VERSION_LINE`                                                       | P08                                                                                                                                                      | Verified |
| CONV-03       | LF output, binary mode                | HLD §5, LLD §5           | `src/output.rs::write_output`                                                         | `help_and_version_end_in_lf_only`, P01–P10 (Windows CI)                                                                                                  | Verified |
| CONV-05       | UTF-8 arguments                       | HLD §5, LLD §3.1         | `src/arguments.rs::to_utf8_bytes`                                                     | `arguments_*`, `run_repeated_named_greeting`                                                                                                             | Verified |
| CONV-06       | Locale-independent                    | HLD §7                   | no environment access                                                                 | Inspection                                                                                                                                               | Verified |
| CONV-07       | Short options, attached values        | LLD §3.3                 | `src/cli.rs::parse_short_cluster`                                                     | `parse_bundled_short_options`, P06                                                                                                                       | Verified |
| CONV-08       | Long options, no abbreviations        | LLD §3.2                 | `src/cli.rs::find_long`                                                               | `parse_name_spellings`, `parse_unrecognized_long_options`, P04                                                                                           | Verified |
| CONV-09       | `--` terminator                       | LLD §3.2                 | `src/cli.rs::parse`                                                                   | `parse_collects_operands`                                                                                                                                | Verified |
| CONV-10       | Operands, `-`                         | LLD §3.2                 | `src/cli.rs::parse`                                                                   | `parse_collects_operands`, P10                                                                                                                           | Verified |
| CONV-11       | Repetition                            | LLD §3.2                 | `src/cli.rs::store`                                                                   | `parse_last_occurrence_wins`                                                                                                                             | Verified |
| CONV-12       | Scan order                            | HLD D4                   | `src/cli.rs::parse`                                                                   | `parse_help_and_version_stop_the_scan`, `parse_error_before_help_wins`                                                                                   | Verified |
| CONV-13       | Exact messages                        | LLD §2.3, §3.6           | `src/diagnostics.rs::UsageError::message`                                             | `diagnostics_match_conv13`, P09, P10                                                                                                                     | Verified |
| CONV-14       | Help hint, exit 2                     | LLD §3.6, §3.8           | `src/diagnostics.rs::UsageError::report`, `src/lib.rs::run`                           | `diagnostics_report_adds_prefix_and_hint`, P09, P10                                                                                                      | Verified |
| CONV-15       | Values unescaped                      | LLD §3.6                 | `src/diagnostics.rs::quoted`                                                          | `diagnostics_echo_values_without_escaping`                                                                                                               | Verified |
| CONV-18       | Exit statuses                         | LLD §3.8                 | `src/lib.rs::run`                                                                     | `run_*`, P01–P10                                                                                                                                         | Verified |
| CONV-22       | `--help`/`--version` take no argument | LLD §3.2                 | `src/cli.rs::parse`                                                                   | `parse_help_and_version_take_no_argument`                                                                                                                | Verified |
| CONV-23       | Help verbatim                         | LLD §3.5                 | `src/greeting.rs::HELP_TEXT`                                                          | `help_and_version_end_in_lf_only`, P07                                                                                                                   | Verified |
| CONV-24       | No crash                              | HLD §6                   | see NFR-03                                                                            | see NFR-03                                                                                                                                               | Verified |
| CONV-25       | Quiet on EPIPE                        | LLD §3.7                 | `src/output.rs::write_output`                                                         | `output_broken_pipe_is_quiet`, `run_output_failure_is_silent`                                                                                            | Verified |
| CONV-26       | No stdin                              | HLD §3.1                 | stdin never opened                                                                    | Inspection                                                                                                                                               | Verified |

CONV-04, CONV-16, CONV-17, CONV-19 … CONV-21 do not apply (no input data, files, or paths).
CONV-27 … CONV-30 describe the test harness and are met by running P01–P10.

## 3. Backward traceability (test → requirement)

| Test                                                                | Type                        | Requirement(s) verified                                |
| :------------------------------------------------------------------ | :-------------------------- | :----------------------------------------------------- |
| P01                                                                 | black-box, public           | FR-01                                                  |
| P02, P03, P04                                                       | black-box, public           | FR-02, CONV-08                                         |
| P05                                                                 | black-box, public           | FR-04                                                  |
| P06                                                                 | black-box, public           | FR-04, CONV-07                                         |
| P07                                                                 | black-box, public           | FR-07, CONV-23                                         |
| P08                                                                 | black-box, public           | FR-08, CONV-02                                         |
| P09                                                                 | black-box, public           | FR-10, FR-11, CONV-13, CONV-14                         |
| P10                                                                 | black-box, public           | FR-09, FR-11, CONV-13                                  |
| `parse_*` (14 tests, `src/cli.rs`)                                  | unit                        | FR-02, FR-06, FR-07, FR-10, CONV-07 … CONV-12, CONV-22 |
| `validate_*` (9 tests, `src/validate.rs`)                           | unit                        | FR-01, FR-03, FR-04, FR-05, FR-06, FR-09               |
| `greeting_*`, `help_and_version_end_in_lf_only` (`src/greeting.rs`) | unit                        | FR-01, FR-02, FR-04, NFR-05, CONV-03, CONV-23          |
| `diagnostics_*` (`src/diagnostics.rs`)                              | unit                        | CONV-13 … CONV-15                                      |
| `output_*` (`src/output.rs`)                                        | unit                        | CONV-03, CONV-25                                       |
| `arguments_*` (`src/arguments.rs`)                                  | unit                        | CONV-05, CONV-24                                       |
| `run_*` (`src/lib.rs`)                                              | unit (end-to-end in memory) | FR-01, FR-02, FR-07, FR-08, FR-11, CONV-18, CONV-25    |

## 4. Coverage summary

| Category                       | Total | Designed | Implemented | Verified |
| :----------------------------- | :---- | :------- | :---------- | :------- |
| User requirements (UR)         | 12    | 12       | 12          | 12       |
| Functional (FR)                | 11    | 11       | 11          | 11       |
| Non-functional (NFR)           | 7     | 7        | 7           | 7        |
| Conventions (CONV, applicable) | 20    | 20       | 20          | 20       |

## 5. Gaps and justifications

None. NFR-02, NFR-04, NFR-07, CONV-06, and CONV-26 are verified by measurement or inspection,
as SRS §7 prescribes.
