# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A single-command CLI (`scaffold-agent-skill`) that lays down a new
*agent skill* directory: a `SKILL.md` with validated frontmatter, a set of
subdirectories, an `evals/` suite, and optionally a Python test-harness project
and a sibling scratch workspace. The generated files encode the
[Agent Skills](https://agentskills.io/specification) authoring conventions, so
**the template strings in `src/skill.rs` are the real product** — changing them
changes what every scaffolded skill looks like.

The Rust crate lives at the repo root's `scaffold-agent-skill/` subdirectory
(the git repo root is one level up). A `docs/` site built with Zensical sits
alongside the crate in the same directory.

## Commands

```bash
cargo build                       # debug build
cargo test                        # unit tests (src/skill.rs) + integration tests (tests/scaffold.rs)
cargo test scaffold_writes        # run one test by name substring
cargo test --test scaffold        # run only the integration test binary
cargo run -- my-skill -d "desc"   # exercise the CLI; see `cargo run -- --help`
cargo clippy --all-targets        # lint
cargo fmt                         # format
```

The only runtime dependency is `clap`; there is no external Rust test framework.

### Docs

```bash
uv run zensical serve             # live preview of docs/ at http://localhost:8000
uv run zensical build             # render to site/ (gitignored build output)
```

`pyproject.toml` / `uv.lock` / `.python-version` exist **only** to pin the
Zensical toolchain (`[tool.uv] package = false` — this is not a Python package).
Prose lives in `docs/`, site config in `zensical.toml`;
`docs/scaffolding-a-skill.md` is the CLI reference page and should be updated
when flags change. `.github/workflows/docs.yml` builds and deploys the site to
GitHub Pages on push to `master`/`main` (the working branch is `dev`).

## Architecture

Three source files, deliberately thin:

- **`src/skill.rs`** — all logic. `ScaffoldOptions` (fully-resolved intent, no
  CLI concepts), `scaffold()` (the one entry point), `validate_name` /
  `validate_description`, and `ScaffoldError`. Every generated file's contents
  come from a `fn *_contents(name, …) -> String` or a `const` string here.
- **`src/lib.rs`** — exists only to expose `skill` as a library so `src/main.rs`
  and `tests/scaffold.rs` share one implementation. Keep new logic testable from
  here rather than in `main.rs`.
- **`src/main.rs`** — clap `Cli` struct plus a `main` that maps flags to
  `ScaffoldOptions` and prints results. Flag *interaction* is resolved here
  before calling `scaffold` (e.g. `--minimal` zeroes `subdirs` and `evals`;
  `--no-evals` only matters when not minimal). `scaffold` itself does not know
  about `--minimal`.

### Invariants to preserve when editing `scaffold()`

- **Validate before touching disk.** Name and description are checked first; a
  bad value must leave the filesystem untouched (tests assert `!tmp.exists()`).
- **Pre-flight existence check.** All target files are collected and checked for
  existence up front; without `--force`, any collision returns
  `AlreadyExists` before any file is written (partial scaffolds are avoided,
  not cleaned up).
- **Directory name == frontmatter `name`.** The skill dir is always
  `parent/<name>`, which is what makes the frontmatter `name` match its
  directory as the skill spec requires. `validate_name` mirrors that spec
  (1–64 chars, `[a-z0-9-]`, no leading/trailing or consecutive `-`).
- **`--python` replaces the `.gitkeep`** in `scripts/` and `tests/` with real
  starter files (`example.py`, `test_skill.py`); every other scaffold subdir
  gets a `.gitkeep` so git tracks it.
- `description` is written into YAML frontmatter via `yaml_quote` (double-quoted,
  escaped) so colons and `#` are safe; `name` is pre-validated to `[a-z0-9-]`
  and therefore needs no JSON/TOML/YAML escaping in any template.

### Tests

`tests/scaffold.rs` is the main coverage: it calls `scaffold()` directly for
tree and file-content assertions and shells out to `CARGO_BIN_EXE_scaffold-agent-skill`
for CLI-only behavior (flag conflicts, `--minimal`). Each test uses a unique
temp dir via an atomic counter and cleans up with `remove_dir_all`. When you add
a generated file or change a template, update the corresponding
`assert!(md.contains(...))` / tree assertions here.

## Other agent configs present

A Codex config (`~/.codex/`) and a Gemini CLI config (`~/.gemini/`) exist on this
machine. Reply `/import` to scan and list what's importable, then
`/import --yes=<digest>` to apply user-level items.
