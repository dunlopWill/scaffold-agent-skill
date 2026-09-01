use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use scaffold_agent_skill::skill::{
    ScaffoldError, ScaffoldOptions, scaffold, validate_description, validate_name,
};

/// A fresh, unique temp directory per call so tests don't collide.
fn unique_tmp() -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("sas-it-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Baseline options under `parent`; override fields as needed per test.
fn opts_in(parent: &Path) -> ScaffoldOptions {
    ScaffoldOptions {
        name: "sample-skill".into(),
        description: "does a thing".into(),
        parent: parent.to_path_buf(),
        subdirs: vec![
            "references".into(),
            "scripts".into(),
            "tests".into(),
            "".into(),
        ],
        evals: true,
        python: false,
        workspace: true,
        force: false,
    }
}

#[test]
fn accepts_kebab_case() {
    assert!(validate_name("my-skill"));
    assert!(validate_name("pdf-processing"));
    assert!(validate_name("skill2"));
    assert!(validate_name("a"));
}

#[test]
fn rejects_bad_names() {
    assert!(!validate_name(""));
    assert!(!validate_name("My-Skill"));
    assert!(!validate_name("-lead"));
    assert!(!validate_name("trail-"));
    assert!(!validate_name("has space"));
    assert!(!validate_name("under_score"));
    assert!(!validate_name(&"x".repeat(65)));
    assert!(!validate_name("pdf--processing")); // consecutive hyphens

    // The full 64-char length is allowed.
    assert!(validate_name(&"x".repeat(64)));
}

#[test]
fn validates_description() {
    assert!(validate_description("Does a thing when X happens."));
    assert!(validate_description(&"d".repeat(1024)));

    assert!(!validate_description(""));
    assert!(!validate_description("   "));
    assert!(!validate_description(&"d".repeat(1025)));
}

#[test]
fn scaffold_writes_expected_tree() {
    let tmp = unique_tmp();
    let opts = opts_in(&tmp);

    let root = scaffold(&opts).expect("scaffold ok");
    assert!(root.join("SKILL.md").is_file());
    assert!(root.join("references/.gitkeep").is_file());
    assert!(root.join("scripts/.gitkeep").is_file());
    assert!(root.join("tests/.gitkeep").is_file());

    // An empty sibling workspace directory.
    let workspace = tmp.join("sample-skill-workspace");
    assert!(workspace.is_dir());
    assert_eq!(fs::read_dir(&workspace).unwrap().count(), 0);

    let md = fs::read_to_string(root.join("SKILL.md")).unwrap();
    assert!(md.starts_with("---\nname: sample-skill\n"));
    assert!(md.contains("description: \"does a thing\"\n"));
    // Body sections that steer the author toward specifics.
    assert!(md.contains("## Workflow"));
    assert!(md.contains("## When to use"));
    assert!(md.contains("## Gotchas"));
    // Optional fields are documented as commented stubs and the reference block.
    assert!(md.contains("# allowed-tools: "));
    assert!(md.contains("compatibility  optional     Max 500 chars."));

    // Second run without --force is rejected.
    assert!(matches!(
        scaffold(&opts),
        Err(ScaffoldError::AlreadyExists(_))
    ));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn generates_evals_suite() {
    let tmp = unique_tmp();
    let root = scaffold(&opts_in(&tmp)).expect("scaffold ok");

    let evals = root.join("evals/evals.json");
    assert!(evals.is_file());
    let json = fs::read_to_string(&evals).unwrap();
    assert!(json.contains("\"skill\": \"sample-skill\""));
    assert!(json.contains("\"tests\": ["));

    let queries = root.join("evals/eval_queries.json");
    assert!(queries.is_file());
    let queries_json = fs::read_to_string(&queries).unwrap();
    assert!(queries_json.contains("\"should_trigger\": true"));
    assert!(queries_json.contains("\"should_trigger\": false"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn no_evals_skips_the_directory() {
    let tmp = unique_tmp();
    let opts = ScaffoldOptions {
        evals: false,
        ..opts_in(&tmp)
    };

    let root = scaffold(&opts).expect("scaffold ok");
    assert!(root.join("SKILL.md").is_file());
    assert!(!root.join("evals").exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn workspace_can_be_disabled() {
    let tmp = unique_tmp();
    let opts = ScaffoldOptions {
        workspace: false,
        ..opts_in(&tmp)
    };

    let root = scaffold(&opts).expect("scaffold ok");
    assert!(root.join("SKILL.md").is_file());
    assert!(!tmp.join("sample-skill-workspace").exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn generates_python_project() {
    let tmp = unique_tmp();
    let opts = ScaffoldOptions {
        python: true,
        ..opts_in(&tmp)
    };

    let root = scaffold(&opts).expect("scaffold ok");

    let pyproject = fs::read_to_string(root.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("name = \"sample-skill\""));
    assert!(pyproject.contains("[dependency-groups]"));
    assert!(pyproject.contains("dev = [\"pytest\"]"));
    assert!(pyproject.contains("testpaths = [\"tests\"]"));

    // Starter script + test, .gitignore, and the SKILL.md scripts section.
    let example = root.join("scripts/example.py");
    assert!(example.is_file());
    assert!(
        fs::read_to_string(&example)
            .unwrap()
            .contains("# /// script")
    );
    assert!(root.join("tests/test_skill.py").is_file());
    assert!(root.join(".gitignore").is_file());

    let md = fs::read_to_string(root.join("SKILL.md")).unwrap();
    assert!(md.contains("## Available scripts"));
    assert!(md.contains("scripts/example.py"));
    assert!(md.contains("plan -> validate -> execute"));

    // Real starter files replace the placeholder .gitkeep in scripts/ and tests/.
    assert!(!root.join("scripts/.gitkeep").exists());
    assert!(!root.join("tests/.gitkeep").exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&example).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "example.py should be executable");
    }

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn python_scripts_section_absent_without_python() {
    let tmp = unique_tmp();
    let root = scaffold(&opts_in(&tmp)).expect("scaffold ok");

    let md = fs::read_to_string(root.join("SKILL.md")).unwrap();
    assert!(!md.contains("## Available scripts"));
    assert!(root.join("scripts/.gitkeep").is_file());
    assert!(!root.join("pyproject.toml").exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn cli_python_and_minimal_conflict() {
    let tmp = unique_tmp();
    fs::create_dir_all(&tmp).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_scaffold-agent-skill"))
        .args(["clash-skill", "-d", "x", "--python", "--minimal", "-p"])
        .arg(&tmp)
        .status()
        .expect("run binary");
    assert!(!status.success());
    assert!(!tmp.join("clash-skill").exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn cli_minimal_writes_only_skill_md() {
    let tmp = unique_tmp();
    fs::create_dir_all(&tmp).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_scaffold-agent-skill"))
        .args(["minimal-skill", "-d", "does a thing", "--minimal", "-p"])
        .arg(&tmp)
        .status()
        .expect("run binary");
    assert!(status.success());

    let root = tmp.join("minimal-skill");
    let mut entries: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(entries, ["SKILL.md"]);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn frontmatter_quotes_description() {
    let tmp = unique_tmp();
    let opts = ScaffoldOptions {
        name: "demo".into(),
        description: "Use this: when things break #now".into(),
        subdirs: vec![],
        ..opts_in(&tmp)
    };

    let root = scaffold(&opts).expect("scaffold ok");
    let md = fs::read_to_string(root.join("SKILL.md")).unwrap();
    assert!(md.contains("description: \"Use this: when things break #now\"\n"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn rejects_invalid_name_before_touching_disk() {
    let tmp = unique_tmp();
    let opts = ScaffoldOptions {
        name: "Bad_Name".into(),
        ..opts_in(&tmp)
    };

    assert!(matches!(
        scaffold(&opts),
        Err(ScaffoldError::InvalidName(_))
    ));
    assert!(!tmp.exists());
}

#[test]
fn rejects_empty_description_before_touching_disk() {
    let tmp = unique_tmp();
    let opts = ScaffoldOptions {
        description: "   ".into(),
        ..opts_in(&tmp)
    };

    assert!(matches!(
        scaffold(&opts),
        Err(ScaffoldError::InvalidDescription)
    ));
    assert!(!tmp.exists());
}
