use super::{BackendError, BackendResult, SearchResult, SkillBackend};
use std::process::Command;

pub struct NpxSkillsBackend;

impl NpxSkillsBackend {
    fn run_npx(&self, args: &[&str]) -> BackendResult<String> {
        let output = Command::new("npx")
            .args(["skills"])
            .args(args)
            .output()
            .map_err(|e| BackendError {
                message: format!("failed to run npx skills: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError {
                message: format!("npx skills failed: {stderr}"),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl SkillBackend for NpxSkillsBackend {
    fn name(&self) -> &str {
        "npx skills"
    }

    fn install(&self, source: &str, skill_name: &str) -> BackendResult<()> {
        self.run_npx(&["add", source, "--skill", skill_name, "-a", "claude-code"])?;
        Ok(())
    }

    fn update(&self, skill_name: &str) -> BackendResult<()> {
        self.run_npx(&["update", skill_name])?;
        Ok(())
    }

    fn uninstall(&self, skill_name: &str) -> BackendResult<()> {
        self.run_npx(&["remove", skill_name])?;
        Ok(())
    }

    fn search(&self, query: &str) -> BackendResult<Vec<SearchResult>> {
        let output = self.run_npx(&["find", query])?;
        Ok(parse_npx_find_output(&output))
    }
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn parse_npx_find_output(output: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for line in output.lines() {
        let clean = strip_ansi(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() || trimmed.starts_with("Install with") || trimmed.starts_with('└') {
            continue;
        }
        // Format: "owner/repo@skill-name  N installs"
        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let Some(full_name) = parts.first() else {
            continue;
        };
        if !full_name.contains('@') {
            continue;
        }
        let mut split = full_name.splitn(2, '@');
        let repo = split.next().unwrap_or("");
        let skill_name = split.next().unwrap_or("");
        if repo.is_empty() || skill_name.is_empty() {
            continue;
        }
        let description = parts
            .get(1)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        results.push(SearchResult {
            name: skill_name.to_string(),
            repo: repo.to_string(),
            description,
        });
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_name() {
        let backend = NpxSkillsBackend;
        assert_eq!(backend.name(), "npx skills");
    }

    #[test]
    fn parse_npx_output() {
        let output = "\x1b[38;5;102mInstall with\x1b[0m npx skills add <owner/repo@skill>\n\n\x1b[38;5;145manthropics/skills@webapp-testing\x1b[0m \x1b[36m112.7K installs\x1b[0m\n\x1b[38;5;102m└ https://skills.sh/anthropics/skills/webapp-testing\x1b[0m\n\n\x1b[38;5;145msamber/cc-skills-golang@golang-testing\x1b[0m \x1b[36m33.5K installs\x1b[0m\n\x1b[38;5;102m└ https://skills.sh/samber/cc-skills-golang/golang-testing\x1b[0m\n";
        let results = parse_npx_find_output(output);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "webapp-testing");
        assert_eq!(results[0].repo, "anthropics/skills");
        assert_eq!(results[1].name, "golang-testing");
        assert_eq!(results[1].repo, "samber/cc-skills-golang");
    }

    #[test]
    fn strip_ansi_codes() {
        assert_eq!(strip_ansi("\x1b[38;5;145mhello\x1b[0m"), "hello");
        assert_eq!(strip_ansi("no codes here"), "no codes here");
    }
}
