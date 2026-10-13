# 00-hello-world: Standard Terminal Hello World
#
# Project Makefile template. It extends the universal build contract in
# shared/templates/BASE-MAKEFILE, which defines the standard targets:
#
#   make all | install | uninstall | test | check | lint | format | clean | distclean
#   make test-hidden | grade | help
#
# A language implementation repository copies this file next to its sources, sets BIN_DIR,
# fills in each DP_* hook with its toolchain's commands, and points CURRICULUM_ROOT at its
# checkout of sdlc-curriculum (for example a git submodule). Hooks that are left empty make
# their target fail with an explanation. `make help` shows which hooks are defined.
#
# Container-first: goals run in the toolchain container built from the repository's
# Containerfile (podman, then docker, then local; override with DP_RUNTIME). The image needs
# the language toolchain, make, and python3 >= 3.11. Use DP_RUNTIME=local for native builds.

PROJECT := 00-hello-world
TOOLS   := dp-hello

# Path to the sdlc-curriculum checkout (the specs, test suites, and runner live there).
CURRICULUM_ROOT ?= ../sdlc-curriculum

# Rust (cargo): Cargo.toml and Cargo.lock live here; sources and unit tests under src/.
# Cargo's target directory is pinned to ./target so BIN_DIR is stable whatever the user's
# CARGO_TARGET_DIR or build.target-dir says.
CARGO        ?= cargo
TARGET_DIR   := target
CARGO_FLAGS  := --locked --target-dir $(TARGET_DIR)

# Directory that contains the built executables after `make all`.
BIN_DIR := $(TARGET_DIR)/release

# Language-specific commands. Use $(TOOLS), $(EXE), $(PREFIX), and $(BINDIR) as needed.
DP_BUILD     = $(CARGO) build --release $(CARGO_FLAGS)
define DP_INSTALL
mkdir -p "$(BINDIR)"
cp "$(BIN_DIR)/$(TOOLS)$(EXE)" "$(BINDIR)/$(TOOLS)$(EXE)"
endef
DP_UNINSTALL = rm -f "$(BINDIR)/$(TOOLS)$(EXE)"
DP_UNIT_TEST = $(CARGO) test $(CARGO_FLAGS)
define DP_LINT
$(CARGO) fmt --check
$(CARGO) clippy --all-targets $(CARGO_FLAGS) -- -D warnings
endef
DP_FORMAT    = $(CARGO) fmt
DP_CLEAN     = $(CARGO) clean --target-dir $(TARGET_DIR)
DP_DISTCLEAN = rm -rf $(TARGET_DIR)

include $(CURRICULUM_ROOT)/shared/templates/BASE-MAKEFILE
