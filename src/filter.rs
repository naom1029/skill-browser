use crate::model::SourceType;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceFilter {
    All,
    GhSkill,
    NpxSkills,
    LocalOnly,
}

impl SourceFilter {
    pub fn next(self) -> Self {
        match self {
            SourceFilter::All => SourceFilter::GhSkill,
            SourceFilter::GhSkill => SourceFilter::NpxSkills,
            SourceFilter::NpxSkills => SourceFilter::LocalOnly,
            SourceFilter::LocalOnly => SourceFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SourceFilter::All => "all",
            SourceFilter::GhSkill => "gh",
            SourceFilter::NpxSkills => "npx",
            SourceFilter::LocalOnly => "local",
        }
    }

    pub fn matches(self, source: &SourceType) -> bool {
        match self {
            SourceFilter::All => true,
            SourceFilter::GhSkill => matches!(source, SourceType::GhSkill),
            SourceFilter::NpxSkills => matches!(source, SourceType::NpxSkills),
            SourceFilter::LocalOnly => matches!(source, SourceType::LocalOnly),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_through_filters_in_order() {
        let filter = SourceFilter::All;
        assert_eq!(filter.next(), SourceFilter::GhSkill);
        assert_eq!(filter.next().next(), SourceFilter::NpxSkills);
        assert_eq!(filter.next().next().next(), SourceFilter::LocalOnly);
        assert_eq!(filter.next().next().next().next(), SourceFilter::All);
    }

    #[test]
    fn labels_are_correct() {
        assert_eq!(SourceFilter::All.label(), "all");
        assert_eq!(SourceFilter::GhSkill.label(), "gh");
        assert_eq!(SourceFilter::NpxSkills.label(), "npx");
        assert_eq!(SourceFilter::LocalOnly.label(), "local");
    }

    #[test]
    fn all_filter_matches_every_source() {
        let all = SourceFilter::All;
        assert!(all.matches(&SourceType::GhSkill));
        assert!(all.matches(&SourceType::Plugin));
        assert!(all.matches(&SourceType::NpxSkills));
        assert!(all.matches(&SourceType::LocalOnly));
    }

    #[test]
    fn gh_filter_matches_only_gh_skill() {
        let gh = SourceFilter::GhSkill;
        assert!(gh.matches(&SourceType::GhSkill));
        assert!(!gh.matches(&SourceType::NpxSkills));
        assert!(!gh.matches(&SourceType::LocalOnly));
        assert!(!gh.matches(&SourceType::Plugin));
    }

    #[test]
    fn npx_filter_matches_only_npx_skills() {
        let npx = SourceFilter::NpxSkills;
        assert!(npx.matches(&SourceType::NpxSkills));
        assert!(!npx.matches(&SourceType::GhSkill));
        assert!(!npx.matches(&SourceType::LocalOnly));
    }

    #[test]
    fn local_filter_matches_only_local() {
        let local = SourceFilter::LocalOnly;
        assert!(local.matches(&SourceType::LocalOnly));
        assert!(!local.matches(&SourceType::GhSkill));
        assert!(!local.matches(&SourceType::NpxSkills));
    }
}
