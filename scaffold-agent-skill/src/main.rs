use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use scaffold_agent_skill::skill::{ScaffoldOptions, scaffold};

/// Scaffold a new Claude Code agent skill directory.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Skill name (1-64 chars, lowercase alphanumeric and hyphens, no leading/trailing or
    /// consecutive hyphens); becomes the directory name.
    name: String,

    /// One-line description written to the SKILL.md frontmatter.
    #[arg(
        short,
        long,
        default_value = "TODO: what this skill does, and the user phrasings that should trigger it."
    )]
    description: String,

    /// Parent directory in which the skill directory is created.
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// Comma-separated subdirectories to create inside the skill.
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "references,scripts,tests,assets"
    )]
    dirs: Vec<String>,

    /// Do not generate the evals/ directory and evals.json.
    #[arg(long = "no-evals")]
    no_evals: bool,

    /// Only write SKILL.md: no subdirectories and no evals/ (overrides --dirs).
    #[arg(long)]
    minimal: bool,

    /// Also generate pyproject.toml (pytest + dev dependency-group) and tests/test_skill.py.
    #[arg(long, conflicts_with = "minimal")]
    python: bool,

    /// Do not create the sibling <name>-workspace/ directory.
    #[arg(long = "no-workspace")]
    no_workspace: bool,

    /// Overwrite existing files (SKILL.md, evals/evals.json) instead of erroring.
    #[arg(short, long)]
    force: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let (subdirs, evals) = if cli.minimal {
        (Vec::new(), false)
    } else {
        (cli.dirs, !cli.no_evals)
    };
    let opts = ScaffoldOptions {
        name: cli.name,
        description: cli.description,
        parent: cli.path,
        subdirs,
        evals,
        python: cli.python,
        workspace: !cli.no_workspace,
        force: cli.force,
    };

    match scaffold(&opts) {
        Ok(root) => {
            println!("Created skill '{}' at {}", opts.name, root.display());
            if opts.workspace {
                let workspace = opts.parent.join(format!("{}-workspace", opts.name));
                println!("Created workspace at {}", workspace.display());
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
