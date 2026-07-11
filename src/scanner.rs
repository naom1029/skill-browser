use crate::model::{Scope, Skill, SourceType};
use crate::parser::parse_skill_md;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct InstalledPlugins {
    #[allow(dead_code)]
    version: u32,
    plugins: HashMap<String, Vec<PluginInstallEntry>>,
}

#[derive(Debug, Deserialize)]
struct PluginInstallEntry {
    #[allow(dead_code)]
    scope: String,
    #[serde(rename = "installPath")]
    install_path: String,
    version: String,
}

pub struct ScanConfig {
    pub home_dir: PathBuf,
    pub project_dir: Option<PathBuf>,
}

pub fn scan_skills(config: &ScanConfig) -> Vec<Skill> {
    let mut skills = Vec::new();

    // User scope
    let claude_skills = config.home_dir.join(".claude").join("skills");
    skills.extend(scan_directory(&claude_skills, SourceType::GhSkill, Scope::User));

    let agents_skills = config.home_dir.join(".agents").join("skills");
    skills.extend(scan_directory(&agents_skills, SourceType::NpxSkills, Scope::User));

    // Plugin skills (nested: ~/.claude/plugins/cache/**/skills/*/)
    let plugin_cache = config.home_dir.join(".claude").join("plugins").join("cache");
    scan_plugin_skills(&config.home_dir, &plugin_cache, &mut skills);

    // Project scope
    if let Some(ref project) = config.project_dir {
        let project_claude = project.join(".claude").join("skills");
        skills.extend(scan_directory(&project_claude, SourceType::LocalOnly, Scope::Project));

        let project_agents = project.join(".agents").join("skills");
        skills.extend(scan_directory(&project_agents, SourceType::LocalOnly, Scope::Project));

        let project_github = project.join(".github").join("skills");
        skills.extend(scan_directory(&project_github, SourceType::LocalOnly, Scope::Project));
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

fn scan_plugin_skills(home_dir: &Path, cache_dir: &Path, skills: &mut Vec<Skill>) {
    let installed_plugins_path = home_dir
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json");

    if let Ok(content) = std::fs::read_to_string(&installed_plugins_path)
        && let Ok(installed) = serde_json::from_str::<InstalledPlugins>(&content)
    {
        scan_installed_plugin_skills(&installed, skills);
        return;
    }

    scan_plugin_skills_recursive(cache_dir, skills);
}

fn scan_installed_plugin_skills(installed: &InstalledPlugins, skills: &mut Vec<Skill>) {
    for entries in installed.plugins.values() {
        for entry in entries {
            let skills_dir = Path::new(&entry.install_path).join("skills");
            if skills_dir.is_dir() {
                let mut found =
                    scan_directory(&skills_dir, SourceType::Plugin, Scope::User);
                for skill in &mut found {
                    skill.version = Some(entry.version.clone());
                }
                skills.extend(found);
            }
        }
    }
}

fn scan_plugin_skills_recursive(cache_dir: &Path, skills: &mut Vec<Skill>) {
    let Ok(marketplaces) = std::fs::read_dir(cache_dir) else { return };
    for marketplace in marketplaces.flatten() {
        let Ok(plugins) = std::fs::read_dir(marketplace.path()) else { continue };
        for plugin in plugins.flatten() {
            let Ok(versions) = std::fs::read_dir(plugin.path()) else { continue };
            for version in versions.flatten() {
                let skills_dir = version.path().join("skills");
                if skills_dir.is_dir() {
                    skills.extend(scan_directory(
                        &skills_dir,
                        SourceType::Plugin,
                        Scope::User,
                    ));
                }
            }
        }
    }
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

        skills.push(Skill {
            name,
            source: source.clone(),
            scope: scope.clone(),
            path: path.clone(),
            description,
            agents: vec![],
            version: None,
            resources,
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

fn collect_resources(skill_dir: &Path) -> Vec<PathBuf> {
    let mut resources = Vec::new();
    fn walk(dir: &Path, resources: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
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
            format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n\nBody content."),
        ).unwrap();
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
        ).unwrap();
        fs::write(skill_dir.join("extra.md"), "extra content").unwrap();
        fs::write(skill_dir.join("script.sh"), "#!/bin/sh\necho hi").unwrap();
        let refs = skill_dir.join("references");
        fs::create_dir_all(&refs).unwrap();
        fs::write(refs.join("ref1.md"), "ref content").unwrap();

        let skills = scan_directory(tmp.path(), SourceType::GhSkill, Scope::User);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].resources.len(), 3); // extra.md + script.sh + references/ref1.md
    }

    #[test]
    fn scan_plugin_skills_falls_back_to_recursive_walk_without_installed_plugins_json() {
        let tmp = tempfile::tempdir().unwrap();
        let home_dir = tmp.path();
        let cache_dir = home_dir.join(".claude").join("plugins").join("cache");
        let versioned_skills_dir = cache_dir
            .join("marketplace")
            .join("superpowers")
            .join("6.1.1")
            .join("skills");
        create_test_skill(&versioned_skills_dir, "brainstorming", "Brainstorm");

        let mut skills = Vec::new();
        scan_plugin_skills(home_dir, &cache_dir, &mut skills);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "brainstorming");
        assert_eq!(skills[0].version, None);
    }

    #[test]
    fn scan_plugin_skills_dedups_via_installed_plugins_json() {
        let tmp = tempfile::tempdir().unwrap();
        let home_dir = tmp.path();
        let cache_dir = home_dir.join(".claude").join("plugins").join("cache");

        // Two cached versions of the same plugin's skill.
        let old_version_dir = cache_dir
            .join("marketplace")
            .join("superpowers")
            .join("6.0.3")
            .join("skills");
        create_test_skill(&old_version_dir, "brainstorming", "Old brainstorm");

        let new_version_dir = cache_dir
            .join("marketplace")
            .join("superpowers")
            .join("6.1.1")
            .join("skills");
        create_test_skill(&new_version_dir, "brainstorming", "New brainstorm");

        let plugins_dir = home_dir.join(".claude").join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        let install_path = new_version_dir
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let installed_json = format!(
            r#"{{
  "version": 2,
  "plugins": {{
    "superpowers@marketplace": [{{
      "scope": "user",
      "installPath": "{install_path}",
      "version": "6.1.1"
    }}]
  }}
}}"#
        );
        fs::write(plugins_dir.join("installed_plugins.json"), installed_json).unwrap();

        let mut skills = Vec::new();
        scan_plugin_skills(home_dir, &cache_dir, &mut skills);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "brainstorming");
        assert_eq!(skills[0].description, "New brainstorm");
        assert_eq!(skills[0].version, Some("6.1.1".to_string()));
    }
}
