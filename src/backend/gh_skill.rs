use super::{BackendError, BackendResult, SearchResult, SkillBackend};
use std::process::Command;

#[derive(serde::Deserialize)]
struct GhSearchItem {
    #[serde(rename = "skillName")]
    skill_name: String,
    repo: String,
    description: String,
}

pub struct GhSkillBackend;

impl GhSkillBackend {
    fn run_gh(&self, args: &[&str]) -> BackendResult<String> {
        let output = Command::new("gh")
            .args(["skill"])
            .args(args)
            .output()
            .map_err(|e| BackendError {
                message: format!("failed to run gh skill: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError {
                message: format!("gh skill failed: {stderr}"),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl GhSkillBackend {
    pub fn search_raw(&self, query: &str) -> BackendResult<String> {
        self.run_gh(&["search", query, "--json", "skillName,repo,description,stars"])
    }
}

impl SkillBackend for GhSkillBackend {
    fn name(&self) -> &str {
        "gh skill"
    }

    fn install(&self, source: &str, skill_name: &str) -> BackendResult<()> {
        self.run_gh(&["install", source, skill_name, "--agent", "claude-code"])?;
        Ok(())
    }

    fn update(&self, skill_name: &str) -> BackendResult<()> {
        self.run_gh(&["update", skill_name])?;
        Ok(())
    }

    fn uninstall(&self, skill_name: &str) -> BackendResult<()> {
        self.run_gh(&["uninstall", skill_name])?;
        Ok(())
    }

    fn search(&self, query: &str) -> BackendResult<Vec<SearchResult>> {
        let output = self.run_gh(&["search", query, "--json", "skillName,repo,description"])?;
        let items: Vec<GhSearchItem> = serde_json::from_str(&output).map_err(|e| BackendError {
            message: format!("failed to parse search results: {e}"),
        })?;
        Ok(items
            .into_iter()
            .map(|item| SearchResult {
                name: item.skill_name,
                repo: item.repo,
                description: item.description,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_name() {
        let backend = GhSkillBackend;
        assert_eq!(backend.name(), "gh skill");
    }
}
