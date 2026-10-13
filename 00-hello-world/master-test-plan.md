# Master Test Plan (MTP) — Standard Terminal Hello World

| Field | Value |
| :-- | :-- |
| Project | `00-hello-world` |
| Tools | dp-hello |
| Implementation language | rs (Rust 1.99, edition 2024) |
| Author | Brandon Perkins |
| Version / date | 1.0 — 2026-10-13 |
| Standard | Structure adapted from IEEE 829 / ISO/IEC/IEEE 29119-3 |

## 1. Introduction and scope

### 1.1 Objectives
Tested enough means: every SRS acceptance criterion (FR-01 … FR-11) is exercised by a unit test
or a black-box test; the public and hidden suites pass byte-for-byte on Linux, macOS, and
Windows; `make lint` (clippy `-D warnings`, `cargo fmt --check`) is clean; and every RTM row is
Verified.

### 1.2 In scope / out of scope
| In scope | Out of scope |
| :-- | :-- |
| Every FR/NFR in the SRS and the applicable CONV rules, on Linux, macOS, and Windows | Terminal rendering of non-ASCII names (a console concern, not a byte-output one) |
| Build, install, and uninstall through the Makefile contract | CONV rules about files, paths, and stdin data (not applicable) |

## 2. Test items

| Item | Version / location |
| :-- | :-- |
| dp-hello | `00-hello-world/src/` on branch `feat/00-hello-world` |
| Public black-box suite | `sdlc-curriculum/projects/00-hello-world/tests/public/` (10 tests) |
| Hidden black-box suite | `sdlc-curriculum/projects/00-hello-world/tests/private/` (10 tests, run by CI) |

## 3. Test levels and approach

| Level | Purpose | Technique(s) | Tooling | Owner |
| :-- | :-- | :-- | :-- | :-- |
| Unit | Each module in isolation | Equivalence partitioning, boundary-value analysis | `cargo test` (`#[cfg(test)]` modules) | Developer |
| Integration | Parser → validation → greeting → output, in memory | Scenario-based | `run_*` tests in `src/lib.rs` | Developer |
| System (black-box) | The specified CLI behavior, byte-exact | Specification-based | `make test` → `run-tests.py` | Developer |
| Acceptance | Hidden suite on every OS | — | CI `make check` (`test-hidden`) | CI |
| Static analysis | Lint and formatting | — | `make lint` (clippy, rustfmt) | Developer / CI |

## 4. Test design

### 4.1 Equivalence partitions and boundaries

| Input | Valid partitions | Invalid partitions | Boundaries |
| :-- | :-- | :-- | :-- |
| NAME | Any non-empty bytes: ASCII, spaces, non-ASCII (`Zoë`), leading `-` | Empty | Length 0 / 1 |
| COUNT | ASCII digits with value 1–1000, leading zeros | Empty, non-digits, signs, spaces, non-ASCII digits, 0, > 1000, huge | 0, 1, 1000, 1001, `000`, `0001000` |
| Option syntax | `-nX`, `-n X`, `--name=X`, `--name X`, repeats | Unknown short/long, abbreviations, missing values, `--help=x` | Value at end of argv |
| Operands | None | Any token not an option, anything after `--` | `-` alone |
| Arguments (encoding) | Valid UTF-8 | Invalid UTF-8 (Unix), unpaired surrogates (Windows) | — |

### 4.2 Additional test cases (beyond the supplied suite)

| ID | Requirement(s) | Preconditions | Steps / command | Expected result | Level |
| :-- | :-- | :-- | :-- | :-- | :-- |
| T-01 | FR-04 | — | `validate_count_boundaries` | 0 and 1001 rejected; 1 and 1000 accepted | unit |
| T-02 | FR-04 | — | `validate_count_leading_zeros` | `007` → 7; `000` rejected | unit |
| T-03 | FR-05 | — | `validate_count_non_digits` | `-1`, `+1`, ` 1`, `1.0`, `٣` all rejected | unit |
| T-04 | FR-03 | — | `validate_empty_name` | `''` rejected for `-n` and `--name`; `' '` accepted | unit |
| T-05 | FR-06 | — | `validate_only_winning_values` | `-c 0 -c 2 --name= -nB` greets B twice | unit |
| T-06 | CONV-07 | — | `parse_bundled_short_options` | `-c2 -nKen`; `-cn` sets count `n` | unit |
| T-07 | FR-02 | — | `parse_name_may_start_with_dash` | `--name=-x`, `-n-x`, `--name -x` accepted | unit |
| T-08 | CONV-09, FR-09 | — | `parse_collects_operands` | `-- -n x` → operands; `-` is an operand | unit |
| T-09 | CONV-12 | — | `parse_help_and_version_stop_the_scan`, `parse_error_before_help_wins` | `--help extra -x` → help; `-x --help` → error | unit |
| T-10 | FR-10, CONV-08 | — | `parse_unrecognized_long_options` | `--nam`, `--NAME`, `--nam=Ada`, `--=x` echoed as given | unit |
| T-11 | FR-10 | — | `parse_missing_arguments`, `parse_help_and_version_take_no_argument` | Exact CONV-13 variants | unit |
| T-12 | CONV-24 | — | `parse_unknown_short_option_with_invalid_utf8`, `arguments_keep_invalid_utf8_bytes_on_unix` | No panic; bytes echoed | unit |
| T-13 | CONV-05 | Windows | `arguments_replace_unpaired_surrogates_on_windows` | U+FFFD, no panic | unit |
| T-14 | CONV-25 | — | `output_broken_pipe_is_quiet`, `run_output_failure_is_silent`; manual `dp-hello -c 1000 \| head -1` | No diagnostic, exit 0 | unit / system |
| T-15 | FR-11 | — | `run_usage_error_writes_only_stderr` | stdout empty, exit 2 | integration |

### 4.3 Negative and robustness testing
Invalid options, values, and operands are covered by T-03 … T-11 and P09/P10. Invalid argument
encodings are covered by T-12/T-13. Output failure (broken pipe, other I/O errors) is simulated
with failing `Write` implementations (T-14). The tool reads no files or stdin, so missing files,
permissions, and large inputs do not apply.

### 4.4 Performance and resource testing
NFR-02 (< 100 ms, < 32 MiB): `/usr/bin/time -f "%e s %M KiB" target/release/dp-hello -c 1000 >
/dev/null` on Linux measured 0.00 s and 1.9 MiB maximum RSS. CI runs the black-box suites on
all three OSes as a smoke check.

## 5. Test environments

| Environment | OS / version | Toolchain | How provisioned |
| :-- | :-- | :-- | :-- |
| Local (container) | Debian trixie (`rust:1.99-slim-trixie`) | Rust 1.99, clippy, rustfmt, Python 3 | `make check` (Podman, `Containerfile`) |
| Local (native) | Fedora 44 | Rust (Fedora package) | `DP_RUNTIME=local make test` |
| CI | ubuntu-latest, macos-latest, windows-latest | rustup 1.99 + clippy + rustfmt, Python 3.12 | `.github/workflows/ci.yml` (`make check roundtrip`) |

## 6. Entry and exit criteria

- **Entry:** HLD and LLD complete; `make all` succeeds; `make lint` is clean.
- **Exit:** every unit test passes; P01–P10 and the hidden suite pass in the container and on
  all three OSes; `make roundtrip` passes natively; no open severity-1/2 defects; every RTM row
  is Verified.
- **Suspension / resumption:** suspend when the build or toolchain is broken on any platform;
  resume once `make toolchain` and `make all` pass again.

## 7. Defect management
Defects are recorded as GitHub issues in `dialectic-playground/rs` with the failing command, the
expected and actual bytes, and the OS. Every fix adds a unit test that reproduces the defect,
and CI re-runs the full matrix.

| Severity | Definition |
| :-- | :-- |
| 1 — Critical | Crash, panic, or wrong output on a mainline path |
| 2 — Major | Wrong diagnostic text or exit status; platform-specific byte difference |
| 3 — Minor | Documentation, lint, or build-hygiene issue with no behavior impact |

## 8. Schedule and deliverables

| Milestone | Deliverable | Target date |
| :-- | :-- | :-- |
| Design complete | `hld.md`, `lld.md` | 2026-10-13 |
| Implementation complete | `src/`, unit tests, Makefile hooks | 2026-10-13 |
| Verification complete | RTM all Verified, CI green | 2026-10-13 |

## 9. Risks and contingencies

| Risk | Mitigation |
| :-- | :-- |
| Windows-only failure (encoding, `.exe`, line endings) | CI runs every OS on each push; Windows-specific unit test T-13 |
| clippy/rustfmt missing from a toolchain | `make toolchain` fails early when they are absent |

## 10. Test summary

| Metric | Value |
| :-- | :-- |
| Unit tests (pass / total) | 40 / 40 per platform (41 defined; one Unix-only, one Windows-only) |
| Coverage (line / branch) | Not measured (no coverage tool in the toolchain); every public function has unit tests |
| Public black-box (pass / 10) | 10 / 10 |
| Hidden black-box (pass / 10, from CI) | 10 / 10 |
| Open defects by severity | 0 / 0 / 0 |
