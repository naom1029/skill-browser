use crate::model::{Scope, Skill};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentFilter {
    All,
    ClaudeCode,
    Multi,
}

impl AgentFilter {
    pub fn next(self) -> Self {
        match self {
            AgentFilter::All => AgentFilter::ClaudeCode,
            AgentFilter::ClaudeCode => AgentFilter::Multi,
            AgentFilter::Multi => AgentFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AgentFilter::All => "all",
            AgentFilter::ClaudeCode => "claude-code",
            AgentFilter::Multi => "multi",
        }
    }

    pub fn matches(self, skill: &Skill) -> bool {
        match self {
            AgentFilter::All => true,
            AgentFilter::ClaudeCode => skill.agents.len() <= 1,
            AgentFilter::Multi => skill.agents.len() > 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeFilter {
    All,
    User,
    Project,
}

impl ScopeFilter {
    pub fn next(self) -> Self {
        match self {
            ScopeFilter::All => ScopeFilter::User,
            ScopeFilter::User => ScopeFilter::Project,
            ScopeFilter::Project => ScopeFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ScopeFilter::All => "all",
            ScopeFilter::User => "user",
            ScopeFilter::Project => "proj",
        }
    }

    pub fn matches(self, skill: &Skill) -> bool {
        match self {
            ScopeFilter::All => true,
            ScopeFilter::User => skill.scope == Scope::User,
            ScopeFilter::Project => skill.scope == Scope::Project,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Scope, SourceType};
    use std::path::PathBuf;

    fn make_skill(agents: Vec<&str>, scope: Scope) -> Skill {
        Skill {
            name: "test".to_string(),
            source: SourceType::GhSkill,
            scope,
            path: PathBuf::from("/tmp/test"),
            description: String::new(),
            agents: agents.into_iter().map(String::from).collect(),
            version: None,
            resources: vec![],
            has_scripts: false,
            pinned: false,
        }
    }

    #[test]
    fn agent_filter_cycles_correctly() {
        assert_eq!(AgentFilter::All.next(), AgentFilter::ClaudeCode);
        assert_eq!(AgentFilter::ClaudeCode.next(), AgentFilter::Multi);
        assert_eq!(AgentFilter::Multi.next(), AgentFilter::All);
    }

    #[test]
    fn agent_filter_claude_code_matches_single_agent() {
        let skill = make_skill(vec!["claude-code"], Scope::User);
        assert!(AgentFilter::ClaudeCode.matches(&skill));
        assert!(!AgentFilter::Multi.matches(&skill));
    }

    #[test]
    fn agent_filter_multi_matches_multiple_agents() {
        let skill = make_skill(vec!["claude-code", "codex", "copilot"], Scope::User);
        assert!(AgentFilter::Multi.matches(&skill));
        assert!(!AgentFilter::ClaudeCode.matches(&skill));
    }

    #[test]
    fn scope_filter_cycles_correctly() {
        assert_eq!(ScopeFilter::All.next(), ScopeFilter::User);
        assert_eq!(ScopeFilter::User.next(), ScopeFilter::Project);
        assert_eq!(ScopeFilter::Project.next(), ScopeFilter::All);
    }

    #[test]
    fn scope_filter_user_matches_user_scope() {
        let skill = make_skill(vec![], Scope::User);
        assert!(ScopeFilter::User.matches(&skill));
        assert!(!ScopeFilter::Project.matches(&skill));
    }

    #[test]
    fn scope_filter_project_matches_project_scope() {
        let skill = make_skill(vec![], Scope::Project);
        assert!(ScopeFilter::Project.matches(&skill));
        assert!(!ScopeFilter::User.matches(&skill));
    }

    #[test]
    fn both_filters_combine_as_and() {
        let skill = make_skill(vec!["claude-code"], Scope::User);
        assert!(AgentFilter::ClaudeCode.matches(&skill) && ScopeFilter::User.matches(&skill));
        assert!(!(AgentFilter::Multi.matches(&skill) && ScopeFilter::User.matches(&skill)));
    }
}
