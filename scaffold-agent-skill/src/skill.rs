use std::fs;
use std::path::PathBuf;

/// Everything needed to lay a skill directory down on disk.
pub struct ScaffoldOptions {
    pub name: String,
    pub description: String,
    pub parent: PathBuf,
    pub subdirs: Vec<String>,
    /// Also create `evals/evals.json` with a starter eval suite.
    pub evals: bool,
    /// Also create `pyproject.toml` and `tests/test_skill.py` for a Python project.
    pub python: bool,
    /// Also create an empty sibling `<name>-workspace/` directory.
    pub workspace: bool,
    pub force: bool,
}

#[derive(Debug)]
pub enum ScaffoldError {
    InvalidName(String),
    InvalidDescription,
    AlreadyExists(PathBuf),
    Io(std::io::Error),
}

impl std::fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScaffoldError::InvalidName(name) => write!(
                f,
                "invalid skill name {name:?}: 1-64 characters, lowercase alphanumeric and hyphens \
                 only, no leading/trailing hyphen and no consecutive hyphens"
            ),
            ScaffoldError::InvalidDescription => write!(
                f,
                "invalid description: must be non-empty and at most 1024 characters"
            ),
            ScaffoldError::AlreadyExists(path) => write!(
                f,
                "{} already exists (pass --force to overwrite)",
                path.display()
            ),
            ScaffoldError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ScaffoldError {}

impl From<std::io::Error> for ScaffoldError {
    fn from(err: std::io::Error) -> Self {
        ScaffoldError::Io(err)
    }
}

/// Frontmatter `name`, which must also equal the skill's directory name:
///
/// - 1-64 characters
/// - lowercase alphanumeric (`a-z`, `0-9`) and hyphens only
/// - must not start or end with a hyphen
/// - must not contain consecutive hyphens (`--`)
pub fn validate_name(name: &str) -> bool {
    if name.is_empty() || name.chars().count() > 64 {
        return false;
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Frontmatter `description`: non-empty, max 1024 characters.
pub fn validate_description(description: &str) -> bool {
    !description.trim().is_empty() && description.chars().count() <= 1024
}

/// Create `<parent>/<name>/` with a `SKILL.md` and the requested subdirectories.
/// Returns the path to the created skill directory.
pub fn scaffold(opts: &ScaffoldOptions) -> Result<PathBuf, ScaffoldError> {
    if !validate_name(&opts.name) {
        return Err(ScaffoldError::InvalidName(opts.name.clone()));
    }
    if !validate_description(&opts.description) {
        return Err(ScaffoldError::InvalidDescription);
    }

    // The directory is named after `name`, so the frontmatter `name` always
    // matches the parent directory name, as the spec requires.
    let root = opts.parent.join(&opts.name);
    let workspace = opts.parent.join(format!("{}-workspace", opts.name));
    let skill_md = root.join("SKILL.md");
    let evals_json = root.join("evals").join("evals.json");
    let eval_queries_json = root.join("evals").join("eval_queries.json");
    let pyproject = root.join("pyproject.toml");
    let gitignore = root.join(".gitignore");
    let example_py = root.join("scripts").join("example.py");
    let py_test = root.join("tests").join("test_skill.py");
    let targets = [
        Some(&skill_md),
        opts.evals.then_some(&evals_json),
        opts.evals.then_some(&eval_queries_json),
        opts.python.then_some(&pyproject),
        opts.python.then_some(&gitignore),
        opts.python.then_some(&example_py),
        opts.python.then_some(&py_test),
    ];
    for target in targets.into_iter().flatten() {
        if target.exists() && !opts.force {
            return Err(ScaffoldError::AlreadyExists(target.clone()));
        }
    }

    fs::create_dir_all(&root)?;
    if opts.workspace {
        // A sibling scratch area for the skill; left empty on purpose.
        fs::create_dir_all(&workspace)?;
    }
    for sub in &opts.subdirs {
        let sub = sub.trim();
        if sub.is_empty() {
            continue;
        }
        let dir = root.join(sub);
        fs::create_dir_all(&dir)?;
        // Under --python, `scripts/` and `tests/` get real starter files; every
        // other empty scaffold dir gets a .gitkeep so git tracks it.
        if !(opts.python && (sub == "scripts" || sub == "tests")) {
            fs::write(dir.join(".gitkeep"), b"")?;
        }
    }

    fs::write(
        &skill_md,
        skill_md_contents(&opts.name, &opts.description, opts.python),
    )?;

    if opts.evals {
        fs::create_dir_all(evals_json.parent().expect("evals.json has a parent"))?;
        fs::write(&evals_json, evals_json_contents(&opts.name))?;
        fs::write(&eval_queries_json, eval_queries_json_contents(&opts.name))?;
    }

    if opts.python {
        fs::create_dir_all(example_py.parent().expect("example.py has a parent"))?;
        fs::create_dir_all(py_test.parent().expect("test_skill.py has a parent"))?;

        fs::write(&pyproject, pyproject_toml_contents(&opts.name))?;
        fs::write(&gitignore, PYTHON_GITIGNORE)?;
        fs::write(&example_py, EXAMPLE_PY)?;
        make_executable(&example_py)?;
        fs::write(&py_test, py_test_contents(&opts.name))?;
    }

    Ok(root)
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) -> std::io::Result<()> {
    Ok(())
}

fn skill_md_contents(name: &str, description: &str, python: bool) -> String {
    // Only meaningful under --python, where scripts/example.py is generated.
    let scripts_section = if python {
        "\n## Available scripts\n\
         \n\
         - **`scripts/example.py`** — starter example; replace with your skill's real \
         scripts. Run `uv run scripts/example.py --help` for its interface.\n\
         \n\
         Scripts are self-contained ([PEP 723](https://peps.python.org/pep-0723/) inline \
         dependencies): run them with `uv run scripts/<name>.py`. `pyproject.toml` is only \
         the test harness (`pytest`).\n\
         \n\
         <!-- For batch or destructive work, prefer plan -> validate -> execute:\n\
         1. [ ] Produce a plan (e.g. `uv run scripts/example.py --dry-run INPUT`)\n\
         2. [ ] Validate the plan against a source of truth\n\
         3. [ ] If validation fails, revise the plan and re-validate\n\
         4. [ ] Execute only once validation passes\n\
         5. [ ] Verify the output -->\n"
    } else {
        ""
    };
    // The template has no literal `{`/`}` outside the named placeholders below.
    format!(
        r#"---
name: {name}
description: {desc}
# Optional frontmatter fields — uncomment and edit as needed:
# license: Apache-2.0
# compatibility: "Needs network access; expects `git` on PATH."
# metadata:
#   author: {name}
#   version: "0.1.0"
# allowed-tools: "Read Grep Glob Bash"
---

# {name}

{plain_desc}

<!--
Authoring notes (delete once written):
- Add only what the agent would not already know: project conventions, the exact
  tools/APIs to use, non-obvious edge cases. Skip general knowledge.
- Keep this file under ~500 lines / 5k tokens. Move deep material into
  references/NAME.md and point to it with a load trigger, e.g.
  "Read references/api-errors.md if the API returns a non-200 status."
- Put output-format templates in assets/ and reference them only when needed.
-->

## Workflow

<!-- A reusable procedure, not a one-off answer. Number the steps, give the agent
     one default approach (not a menu), and include a minimal worked example.
     Explain *why* for flexible steps; be prescriptive where the task is fragile. -->

1. TODO: first step — name the exact tool or command to use.
2. TODO: next step.
3. TODO: validate the result before finishing.

```text
TODO: minimal worked example
```

## When to use

TODO: the situations that should trigger this skill, and any it should not.

## Gotchas

<!-- The highest-value part of most skills: environment-specific facts that defy
     reasonable assumptions. Concrete corrections, not general advice. Add one
     every time you have to correct the agent. -->

- TODO e.g. "The `users` table uses soft deletes — queries need `WHERE deleted_at IS NULL`."
{scripts_section}
<!--
Frontmatter fields:
  name           required     1-64 chars; lowercase alphanumeric and hyphens; no leading/trailing or consecutive hyphens; must equal this directory's name.
  description    required     Max 1024 chars, non-empty. Third person: what the skill does + when to use it, naming the phrases a user would say.
  license        optional     License name, or a reference to a bundled license file.
  compatibility  optional     Max 500 chars. Environment requirements: intended product, system packages, network access.
  metadata       optional     Arbitrary map of string keys to string values.
  allowed-tools  experimental Space-separated list of pre-approved tools the skill may use.
-->
"#,
        name = name,
        desc = yaml_quote(description),
        plain_desc = description.trim(),
        scripts_section = scripts_section,
    )
}

/// Emit a double-quoted YAML scalar so colons, `#`, etc. in the description are safe.
fn yaml_quote(value: &str) -> String {
    let escaped = value.trim().replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// A starter eval suite for the skill. `name` is already validated to
/// `[a-z0-9-]+`, so it needs no JSON escaping.
fn evals_json_contents(name: &str) -> String {
    const TEMPLATE: &str = r#"{
  "skill": "__SKILL_NAME__",
  "description": "Eval cases for the __SKILL_NAME__ skill. Fill in the TODOs and add more.",
  "tests": [
    {
      "name": "activates-on-relevant-request",
      "prompt": "TODO: a realistic user message that should trigger this skill.",
      "expected": "The __SKILL_NAME__ skill is invoked and the task is completed correctly."
    },
    {
      "name": "skips-unrelated-request",
      "prompt": "TODO: a user message in a nearby domain that should NOT trigger this skill.",
      "expected": "The __SKILL_NAME__ skill is not invoked."
    }
  ]
}
"#;
    TEMPLATE.replace("__SKILL_NAME__", name)
}

/// Trigger-eval queries: realistic prompts labeled with whether they should
/// activate the skill. `name` is already validated to `[a-z0-9-]+`.
fn eval_queries_json_contents(name: &str) -> String {
    const TEMPLATE: &str = r#"[
  { "query": "TODO: a realistic prompt that SHOULD trigger the __SKILL_NAME__ skill", "should_trigger": true },
  { "query": "TODO: the same intent phrased differently, still SHOULD trigger", "should_trigger": true },
  { "query": "TODO: a request in an adjacent domain that should NOT trigger", "should_trigger": false },
  { "query": "TODO: an unrelated everyday request that should NOT trigger", "should_trigger": false }
]
"#;
    TEMPLATE.replace("__SKILL_NAME__", name)
}

/// A minimal `pyproject.toml` wiring up pytest and a `dev` dependency group.
/// `name` is already validated to `[a-z0-9-]+`, so it needs no TOML escaping.
fn pyproject_toml_contents(name: &str) -> String {
    const TEMPLATE: &str = r#"# Test harness for this skill. Scripts in scripts/ are self-contained
# (PEP 723) and run via `uv run scripts/<name>.py` without this file.

[project]
name = "__SKILL_NAME__"
version = "0.1.0"
requires-python = ">=3.11"

[dependency-groups]
dev = ["pytest"]

[tool.pytest.ini_options]
testpaths = ["tests"]
pythonpath = ["scripts"]

[tool.uv]
package = false
"#;
    TEMPLATE.replace("__SKILL_NAME__", name)
}

/// A starter pytest module that exercises `scripts/example.py` so `pytest` is
/// green on the first run. `name` is already validated to `[a-z0-9-]+`.
fn py_test_contents(name: &str) -> String {
    const TEMPLATE: &str = r#""""Starter tests for the __SKILL_NAME__ skill's scripts.

Run from the skill directory with `pytest` (or `uv run pytest`). Modules in
``scripts/`` are importable here via the ``pythonpath`` setting in pyproject.toml.
"""

import json
import subprocess
import sys
from pathlib import Path

import pytest

import example  # scripts/example.py

SCRIPT = Path(__file__).resolve().parent.parent / "scripts" / "example.py"


def test_summarize_is_pure() -> None:
    assert example.summarize("a b c") == {"lines": 1, "words": 3, "chars": 5}
    assert example.summarize("") == {"lines": 0, "words": 0, "chars": 0}


def test_cli_emits_json(tmp_path: Path) -> None:
    sample = tmp_path / "notes.txt"
    sample.write_text("alpha beta\ngamma\n", encoding="utf-8")

    result = subprocess.run(
        [sys.executable, str(SCRIPT), str(sample)],
        capture_output=True,
        text=True,
        check=True,
    )
    payload = json.loads(result.stdout)
    assert payload["words"] == 3
    assert payload["source"] == str(sample)


def test_cli_missing_input_exits_3() -> None:
    result = subprocess.run(
        [sys.executable, str(SCRIPT), "no-such-file.txt"],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 3
    assert "not found" in result.stderr


@pytest.mark.skip(reason="TODO: add tests for this skill's real scripts")
def test_todo() -> None:
    raise AssertionError("not implemented")
"#;
    TEMPLATE.replace("__SKILL_NAME__", name)
}

/// Files a Python skill project should keep out of version control.
const PYTHON_GITIGNORE: &str = "__pycache__/\n\
    *.py[cod]\n\
    .pytest_cache/\n\
    .ruff_cache/\n\
    .venv/\n";

/// A self-contained (PEP 723) starter script that follows the conventions
/// agent-invoked scripts should follow: no interactive prompts, a useful
/// `--help`, JSON on stdout / diagnostics on stderr, `--dry-run`, `--output`,
/// and distinct exit codes.
const EXAMPLE_PY: &str = r##"#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Summarize a text file as JSON (line / word / char counts).

This is a placeholder. Replace it with your skill's real logic, but keep the
shape: it shows the conventions an agent-invoked script should follow.

Exit codes:
  0  success
  2  usage error (bad or missing arguments; emitted by argparse)
  3  input file not found
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def summarize(text: str) -> dict[str, int]:
    """Pure function -- trivially unit-testable."""
    return {
        "lines": len(text.splitlines()),
        "words": len(text.split()),
        "chars": len(text),
    }


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/example.py",
        description="Summarize a text file (line/word/char counts) as JSON.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Examples:\n"
            "  uv run scripts/example.py notes.txt\n"
            "  cat notes.txt | uv run scripts/example.py -\n"
            "  uv run scripts/example.py --output summary.json notes.txt\n\n"
            "Exit codes: 0 ok, 2 usage error, 3 input not found."
        ),
    )
    parser.add_argument("input", help="path to the input file, or '-' for stdin")
    parser.add_argument(
        "--output",
        metavar="FILE",
        default="-",
        help="write JSON to FILE instead of stdout ('-' means stdout, the default)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="report what would be read, then exit without producing output",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)

    if args.input == "-":
        source, text = "<stdin>", sys.stdin.read()
    else:
        path = Path(args.input)
        if not path.is_file():
            print(f"error: input file not found: {path}", file=sys.stderr)
            return 3
        source, text = str(path), path.read_text(encoding="utf-8")

    if args.dry_run:
        print(f"dry-run: would summarize {source} ({len(text)} bytes)", file=sys.stderr)
        return 0

    rendered = json.dumps({"source": source, **summarize(text)}, indent=2)
    if args.output == "-":
        print(rendered)
    else:
        Path(args.output).write_text(rendered + "\n", encoding="utf-8")
        print(f"wrote {args.output}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
"##;
