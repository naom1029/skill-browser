use crate::model::{Scope, Skill, SourceType};
use crate::parser::parse_skill_md;
use std::path::{Path, PathBuf};

pub struct ScanConfig {
    pub home_dir: PathBuf,
    pub project_dir: Option<PathBuf>,
}

pub fn scan_skills(config: &ScanConfig) -> Vec<Skill> {
    let mut skills = Vec::new();

    // User scope
    let claude_skills = config.home_dir.join(".claude").join("skills");
    skills.extend(scan_directory(
        &claude_skills,
        SourceType::GhSkill,
        Scope::User,
    ));

    let agents_skills = config.home_dir.join(".agents").join("skills");
    skills.extend(scan_directory(
        &agents_skills,
        SourceType::NpxSkills,
        Scope::User,
    ));

    // Project scope
    if let Some(ref project) = config.project_dir {
        let project_claude = project.join(".claude").join("skills");
        skills.extend(scan_directory(
            &project_claude,
            SourceType::LocalOnly,
            Scope::Project,
        ));

        let project_agents = project.join(".agents").join("skills");
        skills.extend(scan_directory(
            &project_agents,
            SourceType::LocalOnly,
            Scope::Project,
        ));

        let project_github = project.join(".github").join("skills");
        skills.extend(scan_directory(
            &project_github,
            SourceType::LocalOnly,
            Scope::Project,
        ));
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

fn scan_directory(dir: &Path, source: SourceType, scope: Scope) -> Vec<Skill> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };

    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&skill_md) else {
            continue;
        };
        let parsed = parse_skill_md(&content);
        let name = parsed
            .frontmatter
            .as_ref()
            .and_then(|fm| fm.name.clone())
            .unwrap_or_else(|| {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
        let description = parsed
            .frontmatter
            .as_ref()
            .and_then(|fm| fm.description.clone())
            .unwrap_or_default();

        let resources = collect_resources(&path);
        let has_scripts = resources_have_scripts(&resources);

        let metadata = parsed
            .frontmatter
            .as_ref()
            .and_then(|fm| fm.metadata.clone());
        let pinned = metadata
            .as_ref()
            .map(|m| m.contains_key("github-ref") || m.contains_key("github-tree-sha"))
            .unwrap_or(false);
        let version = metadata.as_ref().and_then(|m| m.get("github-ref").cloned());

        let agents = agents_for(&source, &scope, &path);

        skills.push(Skill {
            name,
            source: source.clone(),
            scope: scope.clone(),
            path: path.clone(),
            description,
            agents,
            version,
            resources,
            has_scripts,
            pinned,
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

fn resources_have_scripts(resources: &[PathBuf]) -> bool {
    resources.iter().any(|p| {
        let is_non_md = p
            .extension()
            .map(|ext| !ext.eq_ignore_ascii_case("md"))
            .unwrap_or(true);
        let in_scripts_dir = p.components().any(|c| c.as_os_str() == "scripts");
        is_non_md || in_scripts_dir
    })
}

fn agents_for(source: &SourceType, scope: &Scope, path: &Path) -> Vec<String> {
    match source {
        SourceType::GhSkill => vec!["claude-code".to_string()],
        SourceType::NpxSkills => vec![
            "claude-code".to_string(),
            "copilot".to_string(),
            "codex".to_string(),
        ],
        SourceType::Plugin => vec!["claude-code".to_string()],
        SourceType::LocalOnly => {
            let _ = scope;
            if path.components().any(|c| c.as_os_str() == ".claude") {
                vec!["claude-code".to_string()]
            } else if path.components().any(|c| c.as_os_str() == ".agents") {
                vec![
                    "claude-code".to_string(),
                    "copilot".to_string(),
                    "codex".to_string(),
                ]
            } else {
                vec![]
            }
        }
    }
}

fn collect_resources(skill_dir: &Path) -> Vec<PathBuf> {
    let mut resources = Vec::new();
    fn walk(dir: &Path, resources: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, resources);
            } else if path.file_name().map(|f| f != "SKILL.md").unwrap_or(false) {
                resources.push(path);
            }
        }
    }
    walk(skill_dir, &mut resources);
    resources.sort();
    resources
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_skill(dir: &Path, name: &str, description: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n\nBody content."
            ),
        )
        .unwrap();
    }

    #[test]
    fn scans_single_skill_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path().join(".claude").join("skills");
        create_test_skill(&skills_dir, "test-skill", "A test skill");

        let skills = scan_directory(&skills_dir, SourceType::GhSkill, Scope::User);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, "A test skill");
        assert_eq!(skills[0].source, SourceType::GhSkill);
        assert_eq!(skills[0].scope, Scope::User);
    }

    #[test]
    fn scans_multiple_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path().join("skills");
        create_test_skill(&skills_dir, "alpha", "First");
        create_test_skill(&skills_dir, "beta", "Second");

        let skills = scan_directory(&skills_dir, SourceType::GhSkill, Scope::User);
        assert_eq!(skills.len(), 2);
    }

    #[test]
    fn skips_nonexistent_directory() {
        let skills = scan_directory(
            Path::new("/nonexistent/path"),
            SourceType::GhSkill,
            Scope::User,
        );
        assert!(skills.is_empty());
    }

    #[test]
    fn collects_resource_files() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: desc\n---\n\nBody",
        )
        .unwrap();
        fs::write(skill_dir.join("extra.md"), "extra content").unwrap();
        fs::write(skill_dir.join("script.sh"), "#!/bin/sh\necho hi").unwrap();
        let refs = skill_dir.join("references");
        fs::create_dir_all(&refs).unwrap();
        fs::write(refs.join("ref1.md"), "ref content").unwrap();

        let skills = scan_directory(tmp.path(), SourceType::GhSkill, Scope::User);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].resources.len(), 3); // extra.md + script.sh + references/ref1.md
        assert!(skills[0].has_scripts);
    }

    #[test]
    fn detects_no_scripts_when_only_markdown() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join("md-only-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: md-only-skill\ndescription: desc\n---\n\nBody",
        )
        .unwrap();
        fs::write(skill_dir.join("extra.md"), "extra content").unwrap();

        let skills = scan_directory(tmp.path(), SourceType::GhSkill, Scope::User);
        assert_eq!(skills.len(), 1);
        assert!(!skills[0].has_scripts);
    }

    #[test]
    fn detects_pinned_skill_from_metadata() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join("pinned-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: pinned-skill\ndescription: desc\nmetadata:\n  github-repo: owner/repo\n  github-ref: v1.0.0\n  github-tree-sha: abc123\n---\n\nBody",
        )
        .unwrap();

        let skills = scan_directory(tmp.path(), SourceType::GhSkill, Scope::User);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].pinned);
        assert_eq!(skills[0].version.as_deref(), Some("v1.0.0"));
    }

    #[test]
    fn detects_unpinned_skill_without_metadata() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join("unpinned-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: unpinned-skill\ndescription: desc\n---\n\nBody",
        )
        .unwrap();

        let skills = scan_directory(tmp.path(), SourceType::GhSkill, Scope::User);
        assert_eq!(skills.len(), 1);
        assert!(!skills[0].pinned);
    }

    #[test]
    fn assigns_claude_code_agent_for_gh_skill() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path().join(".claude").join("skills");
        create_test_skill(&skills_dir, "gh-agent-skill", "desc");

        let skills = scan_directory(&skills_dir, SourceType::GhSkill, Scope::User);
        assert_eq!(skills[0].agents, vec!["claude-code".to_string()]);
    }

    #[test]
    fn assigns_multi_agent_for_npx_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path().join(".agents").join("skills");
        create_test_skill(&skills_dir, "npx-agent-skill", "desc");

        let skills = scan_directory(&skills_dir, SourceType::NpxSkills, Scope::User);
        assert!(skills[0].agents.contains(&"claude-code".to_string()));
        assert!(skills[0].agents.len() > 1);
    }
}
