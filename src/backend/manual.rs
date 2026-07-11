use super::{BackendError, BackendResult, SearchResult, SkillBackend};

pub struct ManualBackend;

impl SkillBackend for ManualBackend {
    fn name(&self) -> &str {
        "manual"
    }

    fn install(&self, _source: &str, _skill_name: &str) -> BackendResult<()> {
        Err(BackendError {
            message: "Manual skills cannot be installed via CLI".to_string(),
        })
    }

    fn update(&self, _skill_name: &str) -> BackendResult<()> {
        Err(BackendError {
            message: "Manual skills cannot be updated via CLI".to_string(),
        })
    }

    fn uninstall(&self, skill_name: &str) -> BackendResult<()> {
        std::fs::remove_dir_all(skill_name).map_err(|e| BackendError {
            message: format!("failed to remove skill directory: {e}"),
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
        let backend = ManualBackend;
        assert_eq!(backend.name(), "manual");
    }
}
