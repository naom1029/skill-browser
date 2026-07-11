use crate::model::Skill;
use skim::prelude::*;
use std::io::Cursor;
use std::path::Path;
use terminal_size::{Width, terminal_size};
use tuikit::prelude::Key;

pub enum PickerAction {
    BrowseFiles(usize),
    SwitchToGrep,
    CycleFilter,
    Delete(usize),
    Install,
}

pub struct GrepResult {
    pub action: PickerAction,
    pub query: String,
}

pub enum FileAction {
    OpenInEditor(std::path::PathBuf),
}

fn shell_quote(path: &Path) -> String {
    let s = path.to_string_lossy();
    format!("'{}'", s.replace('\'', r"'\''"))
}

const PREVIEW_RATIO: usize = 60;

fn left_pane_width() -> usize {
    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);
    // skim uses right:60%, so left pane is 40% minus border (2 chars)
    (term_width * (100 - PREVIEW_RATIO) / 100).saturating_sub(4)
}

pub fn run_skill_picker(
    skills: &[Skill],
    project_dir: Option<&Path>,
    filter_label: &str,
) -> Option<PickerAction> {
    let line_width = left_pane_width();
    let input = skills
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}\t{}", i, s.display_line_with_width(line_width)))
        .collect::<Vec<_>>()
        .join("\n");

    let current_exe = std::env::current_exe().ok()?;
    let mut preview_cmd = format!("{} preview {{1}}", shell_quote(&current_exe));
    if let Some(dir) = project_dir {
        preview_cmd.push_str(&format!(" --project {}", shell_quote(dir)));
    }

    let header =
        format!("Enter:files G:grep S:filter({filter_label}) N:install X:del D/U:scroll Esc:quit");

    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .preview(Some(&preview_cmd))
        .preview_window(Some("right:60%:wrap"))
        .header(Some(&header))
        .multi(false)
        .delimiter(Some("\t"))
        .expect(Some("ctrl-g,ctrl-s,ctrl-x,ctrl-n".to_owned()))
        .bind(vec![
            "ctrl-d:preview-page-down",
            "ctrl-u:preview-page-up",
            "ctrl-f:preview-page-down",
            "ctrl-b:preview-page-up",
        ])
        .build()
        .ok()?;

    let item_reader_option = SkimItemReaderOption::default()
        .delimiter("\t")
        .with_nth("2..")
        .build();
    let item_reader = SkimItemReader::new(item_reader_option);
    let items = item_reader.of_bufread(Cursor::new(input));

    let output = Skim::run_with(&options, Some(items))?;

    if output.final_key == Key::Ctrl('g') {
        return Some(PickerAction::SwitchToGrep);
    }

    if output.final_key == Key::Ctrl('s') {
        return Some(PickerAction::CycleFilter);
    }

    if output.final_key == Key::Ctrl('n') {
        return Some(PickerAction::Install);
    }

    if output.final_key == Key::Ctrl('x') {
        let selected = output.selected_items.first()?;
        let text = selected.output().to_string();
        let idx: usize = text.split('\t').next()?.parse().ok()?;
        return Some(PickerAction::Delete(idx));
    }

    if output.is_abort {
        return None;
    }

    let selected = output.selected_items.first()?;
    let text = selected.output().to_string();
    let idx: usize = text.split('\t').next()?.parse().ok()?;

    Some(PickerAction::BrowseFiles(idx))
}

pub fn run_grep_picker(
    skills: &[Skill],
    project_dir: Option<&Path>,
    initial_query: Option<&str>,
) -> Option<GrepResult> {
    let current_exe = std::env::current_exe().ok()?;

    let mut preview_cmd = format!(
        "{} preview-by-name {{1}} --query {{cq}}",
        shell_quote(&current_exe)
    );
    if let Some(dir) = project_dir {
        preview_cmd.push_str(&format!(" --project {}", shell_quote(dir)));
    }

    let mut grep_cmd = format!("{} grep {{}}", shell_quote(&current_exe));
    if let Some(dir) = project_dir {
        grep_cmd.push_str(&format!(" --project {}", shell_quote(dir)));
    }

    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .preview(Some(&preview_cmd))
        .preview_window(Some("right:60%:wrap"))
        .header(Some("grep> type to search SKILL.md content | Esc:back"))
        .multi(false)
        .prompt(Some("grep> "))
        .cmd(Some(&grep_cmd))
        .cmd_query(initial_query)
        .interactive(true)
        .bind(vec![
            "ctrl-d:preview-page-down",
            "ctrl-u:preview-page-up",
            "ctrl-f:preview-page-down",
            "ctrl-b:preview-page-up",
        ])
        .build()
        .ok()?;

    let output = Skim::run_with(&options, None)?;
    let query = output.query.clone();

    if output.is_abort {
        return None;
    }

    let selected = output.selected_items.first()?;
    let selected_text = selected.output().to_string();
    let selected_name = selected_text.split_whitespace().next().unwrap_or("");

    let idx = skills.iter().position(|s| s.name == selected_name)?;

    Some(GrepResult {
        action: PickerAction::BrowseFiles(idx),
        query,
    })
}

pub struct InstallSelection {
    pub repo: String,
    pub skill: String,
    pub source: String,
}

pub enum InstallResult {
    Selected(InstallSelection),
    EmptyEnter,
    Cancelled,
}

pub fn run_install_picker() -> InstallResult {
    let Some(current_exe) = std::env::current_exe().ok() else {
        return InstallResult::Cancelled;
    };

    let search_cmd = format!("{} search {{}}", shell_quote(&current_exe));

    let exe = shell_quote(&current_exe);
    let preview_cmd = format!(
        "full=$(echo {{}} | cut -d' ' -f1); repo=$(echo \"$full\" | sed 's|/\\([^/]*\\)$| \\1|'); {exe} preview-remote $repo 2>&1 || echo 'preview failed'"
    );

    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .preview(Some(&preview_cmd))
        .preview_window(Some("right:60%:wrap"))
        .header(Some(
            "search> type to search | Enter:install | D/U:scroll | Esc:back",
        ))
        .multi(false)
        .prompt(Some("search> "))
        .cmd(Some(&search_cmd))
        .interactive(true)
        .bind(vec!["ctrl-d:preview-page-down", "ctrl-u:preview-page-up"])
        .build();

    let Some(options) = options.ok() else {
        return InstallResult::Cancelled;
    };

    let Some(output) = Skim::run_with(&options, None) else {
        return InstallResult::Cancelled;
    };

    if output.is_abort {
        return InstallResult::Cancelled;
    }

    let Some(selected) = output.selected_items.first() else {
        return InstallResult::EmptyEnter;
    };

    let text = selected.output().to_string();
    let full_path = text.split_whitespace().next().unwrap_or("").trim();
    let source = if text.contains("[npx]") {
        "npx".to_string()
    } else {
        "gh".to_string()
    };

    let Some(last_slash) = full_path.rfind('/') else {
        return InstallResult::EmptyEnter;
    };
    let repo = full_path[..last_slash].to_string();
    let skill = full_path[last_slash + 1..].to_string();

    InstallResult::Selected(InstallSelection {
        repo,
        skill,
        source,
    })
}

pub fn run_file_browser(skill: &Skill) -> Option<FileAction> {
    let mut files: Vec<std::path::PathBuf> = vec![skill.path.join("SKILL.md")];
    files.extend(skill.resources.iter().cloned());

    // Column layout: 0=index, 1=absolute path (hidden, used by `cat` in the
    // preview so it works regardless of cwd), 2=display path (searched/shown).
    let input = files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let display = f.strip_prefix(&skill.path).unwrap_or(f);
            format!("{}\t{}\t{}", i, f.display(), display.display())
        })
        .collect::<Vec<_>>()
        .join("\n");

    let preview_cmd = "cat {2}".to_string();
    let header = format!("{} | Esc:back | Enter:open in $EDITOR", skill.name);

    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .preview(Some(&preview_cmd))
        .preview_window(Some("right:60%:wrap"))
        .header(Some(&header))
        .multi(false)
        .delimiter(Some("\t"))
        .build()
        .ok()?;

    let item_reader_option = SkimItemReaderOption::default().with_nth("3..").build();
    let item_reader = SkimItemReader::new(item_reader_option);
    let items = item_reader.of_bufread(Cursor::new(input));

    let output = Skim::run_with(&options, Some(items))?;

    if output.is_abort {
        return None;
    }

    let selected = output.selected_items.first()?;
    let text = selected.output().to_string();
    let idx: usize = text.split('\t').next()?.parse().ok()?;

    Some(FileAction::OpenInEditor(files.get(idx)?.clone()))
}
