use super::{BackendError, BackendResult, SearchResult, SkillBackend};

use std::process::Command;

pub struct PluginBackend;

impl PluginBackend {
    fn run_claude_plugin(&self, args: &[&str]) -> BackendResult<String> {
        let output = Command::new("claude")
            .args(["plugin"])
            .args(args)
            .output()
            .map_err(|e| BackendError {
                message: format!("failed to run claude plugin: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError {
                message: format!("claude plugin failed: {stderr}"),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl SkillBackend for PluginBackend {
    fn name(&self) -> &str {
        "claude plugin"
    }

    fn install(
        &self,
        source: &str,
        _skill_name: &str,
        _scope: &str,
        _agent: &str,
    ) -> BackendResult<()> {
        self.run_claude_plugin(&["install", source])?;
        Ok(())
    }

    fn update(&self, skill_name: &str) -> BackendResult<()> {
        self.run_claude_plugin(&["update", skill_name])?;
        Ok(())
    }

    fn uninstall(&self, _skill_name: &str) -> BackendResult<()> {
        Err(BackendError {
            message: "Plugin skills cannot be deleted individually. Use 'claude plugin uninstall <plugin-name>' to remove the entire plugin.".to_string(),
        })
    }

    fn search(&self, _query: &str) -> BackendResult<Vec<SearchResult>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_name() {
        let backend = PluginBackend;
        assert_eq!(backend.name(), "claude plugin");
    }
}
