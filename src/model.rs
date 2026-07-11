use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    GhSkill,
    Plugin,
    NpxSkills,
    LocalOnly,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::GhSkill => write!(f, "gh"),
            SourceType::Plugin => write!(f, "plugin"),
            SourceType::NpxSkills => write!(f, "npx"),
            SourceType::LocalOnly => write!(f, "local"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    User,
    Project,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::User => write!(f, "user"),
            Scope::Project => write!(f, "proj"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub source: SourceType,
    pub scope: Scope,
    pub path: PathBuf,
    pub description: String,
    pub agents: Vec<String>,
    pub version: Option<String>,
    pub resources: Vec<PathBuf>,
}

pub const TAG_WIDTH: usize = 12;
const DEFAULT_NAME_MAX: usize = 28;

impl Skill {
    pub fn display_line_with_width(&self, total_width: usize) -> String {
        let name_max = total_width.saturating_sub(TAG_WIDTH).max(10);
        let tag = format!("{}  {}", self.source, self.scope);
        let name = truncate_with_ellipsis(&self.name, name_max);
        let pad = name_max.saturating_sub(display_width(&name));
        format!("{}{:pad$}{:>tw$}", name, "", tag, pad = pad, tw = TAG_WIDTH)
    }

    pub fn display_line(&self) -> String {
        self.display_line_with_width(DEFAULT_NAME_MAX + TAG_WIDTH)
    }
}

fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else {
        let mut end = max_width - 1;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

fn display_width(s: &str) -> usize {
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_line_right_aligned_tags() {
        let skill = Skill {
            name: "brainstorming".to_string(),
            source: SourceType::Plugin,
            scope: Scope::User,
            path: PathBuf::from("/tmp/test"),
            description: "Test skill".to_string(),
            agents: vec![],
            version: None,
            resources: vec![],
        };
        let line = skill.display_line();
        assert!(line.contains("brainstorming"));
        assert!(line.contains("plugin"));
        assert!(line.contains("user"));
        let width = display_width(&line);
        assert_eq!(width, DEFAULT_NAME_MAX + TAG_WIDTH);
    }

    #[test]
    fn display_line_truncates_long_name() {
        let skill = Skill {
            name: "google-cloud-recipe-networking-observability".to_string(),
            source: SourceType::NpxSkills,
            scope: Scope::User,
            path: PathBuf::from("/tmp/test"),
            description: "Test".to_string(),
            agents: vec![],
            version: None,
            resources: vec![],
        };
        let line = skill.display_line();
        assert!(line.contains("…"));
        assert_eq!(display_width(&line), DEFAULT_NAME_MAX + TAG_WIDTH);
    }

    #[test]
    fn display_line_fixed_width() {
        let short = Skill {
            name: "gh".to_string(),
            source: SourceType::GhSkill,
            scope: Scope::Project,
            path: PathBuf::from("/tmp/test"),
            description: "".to_string(),
            agents: vec![],
            version: None,
            resources: vec![],
        };
        let long = Skill {
            name: "a-very-long-skill-name-that-exceeds-the-max".to_string(),
            source: SourceType::Plugin,
            scope: Scope::User,
            path: PathBuf::from("/tmp/test"),
            description: "".to_string(),
            agents: vec![],
            version: None,
            resources: vec![],
        };
        assert_eq!(
            display_width(&short.display_line()),
            display_width(&long.display_line())
        );
    }

    #[test]
    fn source_type_display() {
        assert_eq!(SourceType::GhSkill.to_string(), "gh");
        assert_eq!(SourceType::Plugin.to_string(), "plugin");
        assert_eq!(SourceType::NpxSkills.to_string(), "npx");
        assert_eq!(SourceType::LocalOnly.to_string(), "local");
    }

    #[test]
    fn scope_display() {
        assert_eq!(Scope::User.to_string(), "user");
        assert_eq!(Scope::Project.to_string(), "proj");
    }
}
