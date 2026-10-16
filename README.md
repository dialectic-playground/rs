<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

**Table of Contents** _generated with [DocToc](https://github.com/thlorenz/doctoc)_

- [rs](#rs)
  - [Rust at a glance](#rust-at-a-glance)
  - [Toolchain](#toolchain)
  - [Layout](#layout)
  - [Development](#development)
  - [Implementing a project](#implementing-a-project)
  - [Contributing](#contributing)
  - [License](#license)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/dialectic-playground/.github/main/assets/rs/dist/logo-dark.svg">
    <img alt="dialectic playground / rs" src="https://raw.githubusercontent.com/dialectic-playground/.github/main/assets/rs/dist/logo.svg" height="96">
  </picture>
</p>

# rs

Rust implementation of the Dialectic Playground SDLC curriculum.

📖 **Documentation:** <https://dialectic-playground.github.io/docs/reference/languages/rs/>

This repository implements the eleven projects of the
[SDLC curriculum](https://github.com/dialectic-playground/sdlc-curriculum) in Rust. Each project is
specified to the byte by the curriculum and verified by its black-box tests on Linux, macOS,
and Windows.

**Status:** Implemented projects: 00-hello-world.

## Rust at a glance

| Aspect                  | Rust                                                                |
| :---------------------- | :------------------------------------------------------------------ |
| File extension          | `.rs`                                                               |
| CLI classification      | Included                                                            |
| Image category          | First-Class                                                         |
| Access to OS primitives | Absolute & Direct (POSIX + memory boundaries)                       |
| Entry-point mechanism   | Compiles directly to machine code; args mapped via std::env::args() |

## Toolchain

|                      |                                                                                              |
| :------------------- | :------------------------------------------------------------------------------------------- |
| Release line         | Rust 1.99                                                                                    |
| Build / package tool | cargo                                                                                        |
| Container image      | `rust:1.99-slim-trixie`, pinned by digest ([`Containerfile`](Containerfile))                 |
| Native CI            | rustup (preinstalled): rustup toolchain install 1.99, plus the clippy and rustfmt components |
| Unit tests           | `cargo test` (built-in test harness; tests sit next to the code in `#[cfg(test)]` modules)   |
| Lint                 | `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`                          |
| Format               | `cargo fmt`                                                                                  |
| Dependencies         | One cargo package per project (`Cargo.toml` + committed `Cargo.lock`); builds use `--locked` |

`make toolchain` verifies that the container and native toolchains are on this release line,
and that clippy and rustfmt are installed. The container image adds them with
`rustup component add clippy rustfmt`; natively, run
`rustup component add clippy rustfmt --toolchain 1.99`.

Each project's `make all` builds a release executable into `<project>/target/release`
(`BIN_DIR`); `make install` copies it to `$(BINDIR)` (default `~/.local/bin`), and
`make distclean` removes the whole `target/` directory.

## Layout

| Path                                  | Contents                                                                                                          |
| :------------------------------------ | :---------------------------------------------------------------------------------------------------------------- |
| `00-hello-world/` … `10-sys-monitor/` | One workspace per curriculum project: `Makefile` (with `DP_*` hooks), HLD, LLD, RTM, master test plan, and `src/` |
| `sdlc-curriculum/`                    | The curriculum, as a git submodule pinned to a release tag                                                        |
| `Makefile`                            | `make check` (repository, toolchain, implemented projects), `toolchain`, `projects`, `roundtrip`                  |
| `Containerfile`, `.devcontainer/`     | The toolchain image and the matching dev container                                                                |
| `tools/dp-run`, `tools/container.mk`  | Shared container tooling (copies of the canonical files in `dialectic-playground/.github`)                        |
| `.github/workflows/ci.yml`            | CI: container and native (Linux, macOS, Windows) `make` targets, shared-tooling check, DCO                        |

## Development

Container-first: `make` targets and `tools/dp-run <command>` run in the toolchain container
via Podman, then Docker, then the local system. `DP_RUNTIME=podman|docker|local` overrides
the choice.

```sh
git clone --recurse-submodules https://github.com/dialectic-playground/rs.git
cd rs
make check                    # repository, toolchain, and every implemented project
DP_RUNTIME=local make check   # the same with your native toolchain
make -C 00-hello-world help   # a project's configuration and hooks
```

## Implementing a project

1. Complete the project's `hld.md` and `lld.md` before writing code.
2. Write the implementation and its unit tests under the project's `src/`, add the
   repository's cargo configuration, and fill in `BIN_DIR` and the `DP_*` hooks in
   the project's `Makefile`.
3. Pass `make -C <project> test` (container) and `DP_RUNTIME=local make -C <project> test`.
4. Add the project directory to `IMPLEMENTED` in the root `Makefile`. CI then builds, lints,
   and tests it in the container and natively on all three operating systems.
5. Keep `rtm.md` and `master-test-plan.md` current.

Specifications: [curriculum reference](https://dialectic-playground.github.io/docs/reference/curriculum/). Language setup:
[Set up a language repository](https://dialectic-playground.github.io/docs/how-to/set-up-a-language-repository/).

## Contributing

See the [organization contributing guide](https://github.com/dialectic-playground/.github/blob/main/CONTRIBUTING.md).
Pull requests are squash-merged, titles follow Conventional Commits, and every commit needs a
DCO sign-off.

## License

[MIT](LICENSE)
