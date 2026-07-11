use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct ParsedSkillMd {
    pub frontmatter: Option<Frontmatter>,
    pub body: String,
}

pub fn parse_skill_md(content: &str) -> ParsedSkillMd {
    if !content.starts_with("---") {
        return ParsedSkillMd {
            frontmatter: None,
            body: content.to_string(),
        };
    }

    let after_first = &content[3..];
    let Some(end) = after_first.find("\n---") else {
        return ParsedSkillMd {
            frontmatter: None,
            body: content.to_string(),
        };
    };

    let yaml_str = &after_first[..end];
    let frontmatter: Option<Frontmatter> = serde_yaml::from_str(yaml_str).ok();

    let body_start = 3 + end + 4; // "---" + yaml + "\n---"
    let body = if body_start < content.len() {
        content[body_start..].trim_start_matches('\n').to_string()
    } else {
        String::new()
    };

    ParsedSkillMd { frontmatter, body }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_and_body() {
        let content = "---\nname: test-skill\ndescription: A test skill\n---\n\n# Body\n\nSome content here.";
        let parsed = parse_skill_md(content);
        let fm = parsed.frontmatter.unwrap();
        assert_eq!(fm.name.unwrap(), "test-skill");
        assert_eq!(fm.description.unwrap(), "A test skill");
        assert!(parsed.body.contains("# Body"));
        assert!(parsed.body.contains("Some content here."));
    }

    #[test]
    fn handles_no_frontmatter() {
        let content = "# Just a markdown file\n\nNo frontmatter here.";
        let parsed = parse_skill_md(content);
        assert!(parsed.frontmatter.is_none());
        assert!(parsed.body.contains("# Just a markdown file"));
    }

    #[test]
    fn handles_empty_description() {
        let content = "---\nname: minimal\n---\n\nBody only.";
        let parsed = parse_skill_md(content);
        let fm = parsed.frontmatter.unwrap();
        assert_eq!(fm.name.unwrap(), "minimal");
        assert!(fm.description.is_none());
    }
}
