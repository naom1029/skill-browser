pub mod gh_skill;
pub mod manual;
pub mod npx_skills;
pub mod plugin;

use crate::model::SourceType;

#[derive(Debug)]
pub struct BackendError {
    pub message: String,
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BackendError {}

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub repo: String,
    pub description: String,
}

impl SearchResult {
    pub fn display_line(&self) -> String {
        format!("{:<30} {}", self.name, self.repo)
    }
}

pub trait SkillBackend {
    fn name(&self) -> &str;
    fn install(&self, source: &str, skill_name: &str) -> BackendResult<()>;
    fn update(&self, skill_name: &str) -> BackendResult<()>;
    fn uninstall(&self, skill_name: &str) -> BackendResult<()>;
    fn search(&self, query: &str) -> BackendResult<Vec<SearchResult>>;
}

pub fn backend_for_source(source: &SourceType) -> Box<dyn SkillBackend> {
    match source {
        SourceType::GhSkill => Box::new(gh_skill::GhSkillBackend),
        SourceType::NpxSkills => Box::new(npx_skills::NpxSkillsBackend),
        SourceType::Plugin => Box::new(plugin::PluginBackend),
        SourceType::LocalOnly => Box::new(manual::ManualBackend),
    }
}
