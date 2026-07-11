use clap::{Parser, Subcommand};
use skill_browser::backend::backend_for_source;
use skill_browser::backend::gh_skill::GhSkillBackend;
use skill_browser::backend::npx_skills::NpxSkillsBackend;
use skill_browser::backend::{SearchResult, SkillBackend};
use skill_browser::filter::{AgentFilter, ScopeFilter};
use skill_browser::highlight::highlight_matches;
use skill_browser::preview::render_preview;
use skill_browser::scanner::{ScanConfig, scan_skills};
use skill_browser::ui::{
    FileAction, GrepResult, InstallResult, InstallSelection, PickerAction, run_file_browser,
    run_grep_picker, run_install_picker, run_skill_picker,
};

#[derive(Parser)]
#[command(
    name = "skill-browser",
    version,
    about = "A TUI for browsing, searching, and managing AI agent skills",
    long_about = "Browse installed skills with fuzzy search, full-text grep, live preview,\n\
                  and one-key install/update/delete. Supports gh skill and npx skills backends.\n\n\
                  Keybindings:\n  \
                  Enter    Browse skill files\n  \
                  Ctrl-G   Grep mode (full-text search)\n  \
                  Ctrl-N   Search & install new skill\n  \
                  Ctrl-R   Update selected skill\n  \
                  Ctrl-A   Cycle agent filter\n  \
                  Ctrl-S   Cycle scope filter\n  \
                  Ctrl-X   Delete selected skill\n  \
                  Ctrl-D/U Preview scroll\n  \
                  Esc      Back / Quit"
)]
struct Cli {
    #[arg(
        long,
        short,
        help = "Project directory to scan for project-scoped skills"
    )]
    project: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(hide = true)]
    Preview {
        index: usize,
        #[arg(long)]
        project: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    Grep {
        query: String,
        #[arg(long)]
        project: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    Search { query: String },
    #[command(hide = true)]
    PreviewRemote { repo: String, skill: String },
    #[command(hide = true)]
    PreviewByName {
        name: String,
        #[arg(long)]
        project: Option<std::path::PathBuf>,
        #[arg(long)]
        query: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Preview { index, project }) => run_preview(index, project),
        Some(Commands::Grep { query, project }) => run_grep(&query, project),
        Some(Commands::Search { query }) => run_search(&query),
        Some(Commands::PreviewRemote { repo, skill }) => run_preview_remote(&repo, &skill),
        Some(Commands::PreviewByName {
            name,
            project,
            query,
        }) => run_preview_by_name(&name, project, query.as_deref()),
        None => run_picker(cli.project),
    }
}

fn run_picker(project: Option<std::path::PathBuf>) {
    let home_dir = dirs::home_dir().expect("cannot determine home directory");
    let project_dir = project.or_else(|| std::env::current_dir().ok());

    let config = ScanConfig {
        home_dir,
        project_dir: project_dir.clone(),
    };

    let mut skills = scan_skills(&config);

    if skills.is_empty() {
        eprintln!("No skills found.");
        return;
    }

    let mut agent_filter = AgentFilter::All;
    let mut scope_filter = ScopeFilter::All;

    loop {
        let filtered: Vec<skill_browser::model::Skill> = skills
            .iter()
            .filter(|s| agent_filter.matches(s) && scope_filter.matches(s))
            .cloned()
            .collect();

        match run_skill_picker(
            &filtered,
            project_dir.as_deref(),
            agent_filter.label(),
            scope_filter.label(),
        ) {
            Some(PickerAction::BrowseFiles(idx)) => {
                open_skill_files(&filtered, idx);
            }
            Some(PickerAction::SwitchToGrep) => {
                while let Some(GrepResult {
                    action: PickerAction::BrowseFiles(idx),
                    ..
                }) = run_grep_picker(&filtered, project_dir.as_deref(), None)
                {
                    open_skill_files(&filtered, idx);
                }
            }
            Some(PickerAction::CycleAgent) => {
                agent_filter = agent_filter.next();
            }
            Some(PickerAction::CycleScope) => {
                scope_filter = scope_filter.next();
            }
            Some(PickerAction::Delete(idx)) => {
                if let Some(skill) = filtered.get(idx) {
                    eprint!("Delete \"{}\"? [y/N] ", skill.name);
                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_ok()
                        && input.trim().eq_ignore_ascii_case("y")
                    {
                        let backend = backend_for_source(&skill.source);
                        match backend.uninstall(&skill.name) {
                            Ok(()) => eprintln!("Deleted skill: {}", skill.name),
                            Err(e) => eprintln!("Failed to delete skill {}: {e}", skill.name),
                        }
                        skills = scan_skills(&config);
                        if skills.is_empty() {
                            eprintln!("No skills found.");
                            break;
                        }
                    }
                }
            }
            Some(PickerAction::Update(idx)) => {
                if let Some(skill) = filtered.get(idx) {
                    let backend = backend_for_source(&skill.source);
                    match backend.update(&skill.name) {
                        Ok(()) => eprintln!("Updated skill: {}", skill.name),
                        Err(e) => eprintln!("Failed to update skill {}: {e}", skill.name),
                    }
                    skills = scan_skills(&config);
                }
            }
            Some(PickerAction::Install) => {
                loop {
                    match run_install_picker() {
                        InstallResult::Selected(InstallSelection {
                            repo,
                            skill,
                            source,
                            scope,
                            agent,
                        }) => {
                            eprintln!(
                                "Installing {skill} from {repo} (via {source}, scope={scope}, agent={agent})..."
                            );
                            let install_result = match source.as_str() {
                                "npx" => NpxSkillsBackend.install(&repo, &skill, &scope, &agent),
                                _ => GhSkillBackend.install(&repo, &skill, &scope, &agent),
                            };
                            match install_result {
                                Ok(()) => {
                                    eprintln!("Installed: {skill}");
                                    skills = scan_skills(&config);
                                }
                                Err(e) => eprintln!("Install failed: {e}"),
                            }
                            break;
                        }
                        InstallResult::EmptyEnter => continue,
                        InstallResult::Cancelled => break,
                    }
                }
                clear_preview_cache();
            }
            None => break,
        }
    }
}

fn open_skill_files(skills: &[skill_browser::model::Skill], idx: usize) {
    let Some(skill) = skills.get(idx) else { return };
    if let Some(FileAction::OpenInEditor(path)) = run_file_browser(skill) {
        let editor_val = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let mut parts = editor_val.split_whitespace();
        let editor_cmd = parts.next().unwrap_or("vi");
        let editor_args: Vec<&str> = parts.collect();
        let _ = std::process::Command::new(editor_cmd)
            .args(&editor_args)
            .arg(&path)
            .status();
    }
}

fn run_search(query: &str) {
    if query.trim().is_empty() {
        return;
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut results: Vec<(SearchResult, &str, Option<u64>)> = Vec::new();

    if let Ok(output) = GhSkillBackend.search_raw(query) {
        #[derive(serde::Deserialize)]
        struct GhItem {
            #[serde(rename = "skillName")]
            skill_name: String,
            repo: String,
            description: String,
            stars: Option<u64>,
        }
        if let Ok(items) = serde_json::from_str::<Vec<GhItem>>(&output) {
            for item in items {
                results.push((
                    SearchResult {
                        name: item.skill_name,
                        repo: item.repo,
                        description: item.description,
                    },
                    "gh",
                    item.stars,
                ));
            }
        }
    }

    if let Ok(npx_results) = NpxSkillsBackend.search(query) {
        for r in npx_results {
            if !results
                .iter()
                .any(|(existing, _, _)| existing.name == r.name && existing.repo == r.repo)
            {
                let desc = r.description.clone();
                results.push((r, "npx", None));
                let _ = desc;
            }
        }
    }

    // Write metadata cache for instant preview
    let dir = cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let meta: Vec<serde_json::Value> = results
        .iter()
        .map(|(r, source, stars)| {
            serde_json::json!({
                "name": r.name,
                "repo": r.repo,
                "description": r.description,
                "source": source,
                "stars": stars,
            })
        })
        .collect();
    let _ = std::fs::write(
        dir.join("results.json"),
        serde_json::to_string(&meta).unwrap_or_default(),
    );

    for (r, source, stars) in &results {
        let stats = match (stars, r.description.contains("installs")) {
            (Some(s), _) => format!("★{s}"),
            _ if !r.description.is_empty() && r.description.ends_with("installs") => {
                r.description.clone()
            }
            _ => String::new(),
        };
        let full_name = format!("{}/{}", r.repo, r.name);
        let tag = format!("[{}] {}", source, stats);
        println!("{:<50} {:>20}", full_name, tag);
    }

    // Background prefetch previews for top 5 results
    let exe = std::env::current_exe().unwrap_or_default();
    for (r, _, _) in results.iter().take(5) {
        let cache_key = format!("{}__{}", r.repo.replace('/', "_"), r.name);
        let cache_file = dir.join(&cache_key);
        if cache_file.exists() {
            continue;
        }
        let exe = exe.clone();
        let repo = r.repo.clone();
        let name = r.name.clone();
        std::thread::spawn(move || {
            let output = std::process::Command::new("gh")
                .args(["skill", "preview", &repo, &name])
                .output();
            if let Ok(out) = output
                && out.status.success()
            {
                let _ = std::fs::write(&cache_file, &out.stdout);
            }
            let _ = exe; // keep exe alive
        });
    }
}

fn cache_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("/tmp/skill-browser-cache")
}

fn clear_preview_cache() {
    let _ = std::fs::remove_dir_all(cache_dir());
}

fn prefetch_nearby(dir: &std::path::Path, current_repo: &str, current_skill: &str) {
    let meta_file = dir.join("results.json");
    let Ok(meta_str) = std::fs::read_to_string(&meta_file) else {
        return;
    };
    let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&meta_str) else {
        return;
    };

    let Some(pos) = items.iter().position(|i| {
        i.get("repo").and_then(|v| v.as_str()) == Some(current_repo)
            && i.get("name").and_then(|v| v.as_str()) == Some(current_skill)
    }) else {
        return;
    };

    let start = pos.saturating_sub(5);
    let end = (pos + 6).min(items.len());

    for item in &items[start..end] {
        let Some(repo) = item.get("repo").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(name) = item.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let cache_key = format!("{}__{}", repo.replace('/', "_"), name);
        let cache_file = dir.join(&cache_key);
        if cache_file.exists() {
            continue;
        }
        let repo = repo.to_string();
        let name = name.to_string();
        let cache_file = cache_file.to_path_buf();
        std::thread::spawn(move || {
            let output = std::process::Command::new("gh")
                .args(["skill", "preview", &repo, &name])
                .output();
            if let Ok(out) = output
                && out.status.success()
            {
                let _ = std::fs::write(&cache_file, &out.stdout);
            }
        });
    }
}

fn run_preview_remote(repo: &str, skill: &str) {
    let dir = cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let cache_key = format!("{}__{}", repo.replace('/', "_"), skill);
    let cache_file = dir.join(&cache_key);

    // Cache hit → instant
    if let Ok(cached) = std::fs::read_to_string(&cache_file) {
        print!("{cached}");
        return;
    }

    // Prefetch nearby items in background
    prefetch_nearby(&dir, repo, skill);

    // Fetch full preview (skim kills this process on cursor move)
    let output = std::process::Command::new("gh")
        .args(["skill", "preview", repo, skill])
        .output();

    if let Ok(out) = output
        && out.status.success()
    {
        let content = String::from_utf8_lossy(&out.stdout);
        let _ = std::fs::write(&cache_file, content.as_bytes());
        print!("{content}");
    }
}

fn run_grep(query: &str, project: Option<std::path::PathBuf>) {
    let home_dir = dirs::home_dir().expect("cannot determine home directory");
    let project_dir = project.or_else(|| std::env::current_dir().ok());
    let config = ScanConfig {
        home_dir,
        project_dir,
    };

    let skills = scan_skills(&config);
    let query_lower = query.to_lowercase();

    let term_width = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .unwrap_or(80);
    let line_width = (term_width * 40 / 100).saturating_sub(4);

    for skill in &skills {
        let haystack_desc = skill.description.to_lowercase();
        let haystack_body = std::fs::read_to_string(skill.path.join("SKILL.md"))
            .unwrap_or_default()
            .to_lowercase();

        if haystack_desc.contains(&query_lower) || haystack_body.contains(&query_lower) {
            println!("{}", skill.display_line_with_width(line_width));
        }
    }
}

fn run_preview_by_name(name: &str, project: Option<std::path::PathBuf>, query: Option<&str>) {
    let home_dir = dirs::home_dir().expect("cannot determine home directory");
    let project_dir = project.or_else(|| std::env::current_dir().ok());
    let config = ScanConfig {
        home_dir,
        project_dir,
    };
    let skills = scan_skills(&config);

    let skill_name = name.replace('…', "");
    if let Some(skill) = skills.iter().find(|s| s.name.starts_with(&skill_name)) {
        let preview = render_preview(skill);
        match query {
            Some(q) if !q.is_empty() => print!("{}", highlight_matches(&preview, q)),
            _ => print!("{}", preview),
        }
    }
}

fn run_preview(index: usize, project: Option<std::path::PathBuf>) {
    let home_dir = dirs::home_dir().expect("cannot determine home directory");
    let project_dir = project.or_else(|| std::env::current_dir().ok());

    let config = ScanConfig {
        home_dir,
        project_dir,
    };

    let skills = scan_skills(&config);

    match skills.get(index) {
        Some(skill) => println!("{}", render_preview(skill)),
        None => {
            eprintln!("No skill at index {index}");
            std::process::exit(1);
        }
    }
}
