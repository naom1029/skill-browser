use crate::model::Skill;
use crate::parser::parse_skill_md;

pub fn render_preview(skill: &Skill) -> String {
    let skill_md_path = skill.path.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    let parsed = parse_skill_md(&content);

    let mut output = String::new();

    // Description header (fixed)
    output.push_str("── Description ──────────────────────────\n");
    if !skill.description.is_empty() {
        output.push_str(&skill.description);
        output.push('\n');
    }
    output.push_str("─────────────────────────────────────────\n\n");

    // Body
    output.push_str(&parsed.body);

    // Footer: resource count
    if !skill.resources.is_empty() {
        let count = skill.resources.len();
        let label = if count == 1 { "file" } else { "files" };
        output.push_str(&format!("\n\n── +{count} {label} (Enter to browse) ──"));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SourceType, Scope};
    use std::path::PathBuf;
    use std::fs;

    #[test]
    fn preview_has_description_header() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: test\ndescription: My great skill\n---\n\n# Heading\n\nBody text.",
        ).unwrap();

        let skill = Skill {
            name: "test".to_string(),
            source: SourceType::GhSkill,
            scope: Scope::User,
            path: skill_dir,
            description: "My great skill".to_string(),
            agents: vec![],
            version: None,
            resources: vec![],
        };

        let preview = render_preview(&skill);
        assert!(preview.starts_with("── Description "));
        assert!(preview.contains("My great skill"));
        assert!(preview.contains("# Heading"));
        assert!(preview.contains("Body text."));
    }

    #[test]
    fn preview_shows_resource_count() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: t\ndescription: d\n---\n\nbody").unwrap();
        fs::write(skill_dir.join("extra.md"), "extra").unwrap();

        let skill = Skill {
            name: "t".to_string(),
            source: SourceType::GhSkill,
            scope: Scope::User,
            path: skill_dir,
            description: "d".to_string(),
            agents: vec![],
            version: None,
            resources: vec![PathBuf::from("extra.md")],
        };

        let preview = render_preview(&skill);
        assert!(preview.contains("1 file"));
    }
}
