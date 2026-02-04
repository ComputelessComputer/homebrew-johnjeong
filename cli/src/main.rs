use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Clone, Copy)]
struct Link {
    label: &'static str,
    url: &'static str,
}

struct AboutData {
    tagline: &'static str,
    links: &'static [Link],
}

struct Post {
    title: String,
    date: String,
    body: String,
    url: String,
    sort_key: String,
}

struct ContentTab {
    name: &'static str,
    description: &'static str,
    posts: Vec<Post>,
}

enum TabData {
    About(AboutData),
    Content(ContentTab),
}

struct HeaderData {
    title: String,
    subtitle: String,
}

struct AppData {
    header: HeaderData,
    tabs: Vec<TabData>,
}

struct AppState {
    tab_index: usize,
    list_index: usize,
    list_scroll: usize,
    content_scroll: usize,
    content_scroll_max: usize,
    status: Option<String>,
}

const ABOUT_LINKS: [Link; 4] = [
    Link {
        label: "LinkedIn",
        url: "https://www.linkedin.com/in/johntopia/",
    },
    Link {
        label: "X (Twitter)",
        url: "https://x.com/computeless",
    },
    Link {
        label: "GitHub",
        url: "https://github.com/ComputelessComputer",
    },
    Link {
        label: "Email",
        url: "mailto:john@hyprnote.com",
    },
];

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("johnjeong {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let (data, status) = build_app_data();
    let mut state = AppState {
        tab_index: 0,
        list_index: 0,
        list_scroll: 0,
        content_scroll: 0,
        content_scroll_max: 0,
        status,
    };

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let result = run_app(&mut stdout, &data, &mut state);

    terminal::disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;

    result
}

fn build_app_data() -> (AppData, Option<String>) {
    let header = load_header_data();
    let content_root = resolve_content_root();
    let mut status = None;

    let essays = content_root
        .as_ref()
        .map(|root| root.join("essays"))
        .and_then(|dir| load_posts(&dir, "https://johnjeong.com/essays", true).ok())
        .unwrap_or_else(|| {
            status = Some(
        "Content directory not found. Set JOHNJEONG_CONTENT_DIR to your part-of-my-brain path."
          .to_string(),
      );
            Vec::new()
        });

    let journals = content_root
        .as_ref()
        .map(|root| root.join("journals"))
        .and_then(|dir| load_posts(&dir, "https://johnjeong.com/journals", false).ok())
        .unwrap_or_default();

    let inspirations = content_root
        .as_ref()
        .map(|root| root.join("inspirations"))
        .and_then(|dir| load_posts(&dir, "https://johnjeong.com/inspirations", false).ok())
        .unwrap_or_default();

    let lessons = content_root
        .as_ref()
        .map(|root| root.join("lessons"))
        .and_then(|dir| load_posts(&dir, "https://johnjeong.com/lessons", false).ok())
        .unwrap_or_default();

    let gallery = content_root
        .as_ref()
        .map(|root| root.join("gallery"))
        .and_then(|dir| load_gallery(&dir).ok())
        .unwrap_or_default();

    let tabs = vec![
        TabData::About(AboutData {
            tagline: "I like simple & intuitive stuff.",
            links: &ABOUT_LINKS,
        }),
        TabData::Content(ContentTab {
            name: "Essays",
            description: "Long-form writing.",
            posts: essays,
        }),
        TabData::Content(ContentTab {
            name: "Daily Logs",
            description: "Daily notes and logs.",
            posts: journals,
        }),
        TabData::Content(ContentTab {
            name: "Inspirations",
            description: "Talks, podcasts, and ideas that shaped me.",
            posts: inspirations,
        }),
        TabData::Content(ContentTab {
            name: "Lessons",
            description: "Learning notes and highlights.",
            posts: lessons,
        }),
        TabData::Content(ContentTab {
            name: "Gallery",
            description: "Photos I took.",
            posts: gallery,
        }),
    ];

    (AppData { header, tabs }, status)
}

fn run_app(stdout: &mut Stdout, data: &AppData, state: &mut AppState) -> io::Result<()> {
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            render(stdout, data, state)?;
            needs_redraw = false;
        }

        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(key, data, state)? {
                        break;
                    }
                    needs_redraw = true;
                }
                Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_key(key: KeyEvent, data: &AppData, state: &mut AppState) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Char('1') => switch_tab(state, 0, data.tabs.len()),
        KeyCode::Char('2') => switch_tab(state, 1, data.tabs.len()),
        KeyCode::Char('3') => switch_tab(state, 2, data.tabs.len()),
        KeyCode::Char('4') => switch_tab(state, 3, data.tabs.len()),
        KeyCode::Char('5') => switch_tab(state, 4, data.tabs.len()),
        KeyCode::Char('6') => switch_tab(state, 5, data.tabs.len()),
        KeyCode::Char('g') => switch_tab(state, 5, data.tabs.len()),
        KeyCode::Up | KeyCode::Char('k') => move_selection(data, state, -1),
        KeyCode::Down | KeyCode::Char('j') => move_selection(data, state, 1),
        KeyCode::PageUp => scroll_content(state, -10),
        KeyCode::PageDown => scroll_content(state, 10),
        KeyCode::Home | KeyCode::Char('t') => state.content_scroll = 0,
        KeyCode::Char('G') => state.content_scroll = state.content_scroll_max,
        KeyCode::Char('o') | KeyCode::Enter => open_selected(data, state),
        _ => {}
    }

    Ok(false)
}

fn switch_tab(state: &mut AppState, index: usize, total: usize) {
    if index >= total {
        return;
    }
    state.tab_index = index;
    state.list_index = 0;
    state.list_scroll = 0;
    state.content_scroll = 0;
    state.content_scroll_max = 0;
    state.status = None;
}

fn move_selection(data: &AppData, state: &mut AppState, delta: i32) {
    let max = list_length(data, state.tab_index);
    if max == 0 {
        state.list_index = 0;
        return;
    }
    let len = max as i32;
    let mut next = state.list_index as i32 + delta;
    if next < 0 {
        next = len - 1;
    } else if next >= len {
        next = 0;
    }
    state.list_index = next as usize;
    state.content_scroll = 0;
    state.content_scroll_max = 0;
}

fn scroll_content(state: &mut AppState, delta: i32) {
    let max = state.content_scroll_max as i32;
    let mut next = state.content_scroll as i32 + delta;
    if next < 0 {
        next = 0;
    } else if next > max {
        next = max;
    }
    state.content_scroll = next as usize;
}

fn open_selected(data: &AppData, state: &mut AppState) {
    match data.tabs.get(state.tab_index) {
        Some(TabData::About(about)) => {
            if let Some(link) = about.links.get(state.list_index) {
                let result = open_url(link.url);
                state.status = Some(match result {
                    Ok(()) => format!("Opened {}", link.label),
                    Err(err) => format!("Failed to open {} ({})", link.label, err),
                });
            }
        }
        Some(TabData::Content(tab)) => {
            if let Some(post) = tab.posts.get(state.list_index) {
                let result = open_url(&post.url);
                state.status = Some(match result {
                    Ok(()) => format!("Opened {}", post.title),
                    Err(err) => format!("Failed to open {} ({})", post.title, err),
                });
            }
        }
        None => {}
    }
}

fn render(stdout: &mut Stdout, data: &AppData, state: &mut AppState) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let max_width = cols.saturating_sub(4) as usize;
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    queue!(
        stdout,
        MoveTo(2, 1),
        SetAttribute(Attribute::Bold),
        Print(&data.header.title),
        SetAttribute(Attribute::Reset)
    )?;

    queue!(
        stdout,
        MoveTo(2, 2),
        SetForegroundColor(Color::DarkGrey),
        Print(&data.header.subtitle),
        ResetColor
    )?;

    render_nav(stdout, data, state, 4)?;

    let content_top = 6;
    match data.tabs.get(state.tab_index) {
        Some(TabData::About(about)) => {
            render_about(stdout, state, about, max_width, rows, content_top)?
        }
        Some(TabData::Content(tab)) => {
            render_content_tab(stdout, state, tab, max_width, rows, content_top)?
        }
        None => {}
    }

    if let Some(status) = &state.status {
        let text = clamp_text(status, max_width);
        queue!(
            stdout,
            MoveTo(2, rows.saturating_sub(3)),
            SetForegroundColor(Color::DarkGrey),
            Print(text),
            ResetColor
        )?;
    }

    queue!(
    stdout,
    MoveTo(2, rows.saturating_sub(2)),
    SetForegroundColor(Color::DarkGrey),
    Print("↑/↓ or j/k move  •  o/enter open  •  pgup/pgdn scroll  •  1-6 tabs (g gallery)  •  q quit"),
    ResetColor
  )?;

    stdout.flush()?;
    Ok(())
}

fn render_nav(stdout: &mut Stdout, data: &AppData, state: &AppState, y: u16) -> io::Result<()> {
    let mut x = 2;
    for (idx, tab) in data.tabs.iter().enumerate() {
        let label = tab_label(idx, tab);
        queue!(stdout, MoveTo(x, y))?;
        if idx == state.tab_index {
            queue!(
                stdout,
                SetAttribute(Attribute::Underlined),
                Print(label),
                SetAttribute(Attribute::Reset)
            )?;
        } else {
            queue!(stdout, Print(label))?;
        }
        x += label.len() as u16 + 3;
    }
    Ok(())
}
fn render_about(
    stdout: &mut Stdout,
    state: &mut AppState,
    about: &AboutData,
    max_width: usize,
    rows: u16,
    content_top: u16,
) -> io::Result<()> {
    let tagline = clamp_text(about.tagline, max_width);
    queue!(stdout, MoveTo(2, content_top), Print(tagline))?;

    let list_y = content_top + 2;
    let list_height = rows.saturating_sub(list_y + 3) as usize;
    state.list_scroll = clamp_scroll(
        state.list_scroll,
        state.list_index,
        list_height,
        about.links.len(),
    );

    for (idx, link) in about
        .links
        .iter()
        .enumerate()
        .skip(state.list_scroll)
        .take(list_height)
    {
        let is_selected = idx == state.list_index;
        let y = list_y + (idx - state.list_scroll) as u16;
        queue!(stdout, MoveTo(4, y))?;
        if is_selected {
            queue!(
                stdout,
                SetForegroundColor(Color::Black),
                SetBackgroundColor(Color::White)
            )?;
        } else {
            queue!(stdout, SetForegroundColor(Color::White))?;
        }
        queue!(
            stdout,
            Print(format!(
                "{} {}",
                if is_selected { "›" } else { " " },
                link.label
            )),
            ResetColor
        )?;
    }

    Ok(())
}

fn render_content_tab(
    stdout: &mut Stdout,
    state: &mut AppState,
    tab: &ContentTab,
    max_width: usize,
    rows: u16,
    content_top: u16,
) -> io::Result<()> {
    queue!(
        stdout,
        MoveTo(2, content_top),
        SetAttribute(Attribute::Bold),
        Print(tab.name),
        SetAttribute(Attribute::Reset)
    )?;

    let description = clamp_text(tab.description, max_width);
    queue!(stdout, MoveTo(2, content_top + 1), Print(description))?;

    let list_x = 2;
    let list_y = content_top + 3;
    let list_width = ((max_width as f32) * 0.33) as usize;
    let list_width = list_width.clamp(24, 38);
    let list_height = rows.saturating_sub(list_y + 3) as usize;

    state.list_scroll = clamp_scroll(
        state.list_scroll,
        state.list_index,
        list_height,
        tab.posts.len(),
    );

    queue!(
        stdout,
        MoveTo(list_x, list_y - 1),
        SetForegroundColor(Color::DarkGrey),
        Print("Posts"),
        ResetColor
    )?;

    if tab.posts.is_empty() {
        queue!(
            stdout,
            MoveTo(list_x, list_y),
            SetForegroundColor(Color::DarkGrey),
            Print("No posts found."),
            ResetColor
        )?;
    } else {
        for (idx, post) in tab
            .posts
            .iter()
            .enumerate()
            .skip(state.list_scroll)
            .take(list_height)
        {
            let is_selected = idx == state.list_index;
            let y = list_y + (idx - state.list_scroll) as u16;
            queue!(stdout, MoveTo(list_x, y))?;
            if is_selected {
                queue!(
                    stdout,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(Color::White)
                )?;
            } else {
                queue!(stdout, SetForegroundColor(Color::White))?;
            }
            let date = if post.date.is_empty() {
                "".to_string()
            } else {
                format!("{} ", format_date(&post.date))
            };
            let label = clamp_text(&format!("{}{}", date, post.title), list_width - 2);
            queue!(
                stdout,
                Print(format!("{} {}", if is_selected { "›" } else { " " }, label)),
                ResetColor
            )?;
        }
    }

    let content_x = (list_x + list_width as u16 + 2).min(max_width as u16);
    let content_width = max_width.saturating_sub(content_x as usize + 1).max(10);

    if let Some(post) = tab.posts.get(state.list_index) {
        let content_top = list_y - 1;
        let mut y = content_top;

        let title = clamp_text(&post.title, content_width);
        queue!(
            stdout,
            MoveTo(content_x, y),
            SetAttribute(Attribute::Bold),
            Print(title),
            SetAttribute(Attribute::Reset)
        )?;
        y += 1;

        if !post.date.is_empty() {
            queue!(
                stdout,
                MoveTo(content_x, y),
                SetForegroundColor(Color::DarkGrey),
                Print(post.date.clone()),
                ResetColor
            )?;
            y += 1;
        }

        let lines = wrap_markdown(&post.body, content_width);
        let available = rows.saturating_sub(y + 2) as usize;
        state.content_scroll_max = lines.len().saturating_sub(available);
        if state.content_scroll > state.content_scroll_max {
            state.content_scroll = state.content_scroll_max;
        }

        for line in lines.iter().skip(state.content_scroll).take(available) {
            queue!(
                stdout,
                MoveTo(content_x, y),
                Print(clamp_text(line, content_width))
            )?;
            y += 1;
            if y >= rows.saturating_sub(2) {
                break;
            }
        }
    } else {
        state.content_scroll_max = 0;
        queue!(
            stdout,
            MoveTo(content_x, list_y),
            SetForegroundColor(Color::DarkGrey),
            Print("Select a post to read."),
            ResetColor
        )?;
    }

    Ok(())
}

fn clamp_scroll(scroll: usize, index: usize, height: usize, total: usize) -> usize {
    if total <= height {
        return 0;
    }
    if index < scroll {
        return index;
    }
    if index >= scroll + height {
        return index.saturating_sub(height.saturating_sub(1));
    }
    scroll.min(total.saturating_sub(height))
}

fn list_length(data: &AppData, tab_index: usize) -> usize {
    match data.tabs.get(tab_index) {
        Some(TabData::About(about)) => about.links.len(),
        Some(TabData::Content(tab)) => tab.posts.len(),
        None => 0,
    }
}

fn tab_name(tab: &TabData) -> &'static str {
    match tab {
        TabData::About(_) => "About",
        TabData::Content(tab) => tab.name,
    }
}

fn tab_label(index: usize, tab: &TabData) -> String {
    format!("{}. {}", index + 1, tab_name(tab))
}

fn resolve_content_root() -> Option<PathBuf> {
    if let Ok(path) = env::var("JOHNJEONG_CONTENT_DIR") {
        let candidate = PathBuf::from(path);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }

    let mut dir = env::current_dir().ok()?;
    for _ in 0..6 {
        let candidate = dir.join("part-of-my-brain");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn load_header_data() -> HeaderData {
    let mut title = env::var("JOHNJEONG_TITLE").unwrap_or_else(|_| "John Jeong".to_string());
    let mut subtitle = env::var("JOHNJEONG_SUBTITLE")
        .unwrap_or_else(|_| "Co-founder & Co-CEO at Hyprnote".to_string());

    if let Some(header_path) = resolve_header_path() {
        if let Ok(contents) = fs::read_to_string(header_path) {
            if let Some(value) = extract_quoted_value(&contents, "title =") {
                title = value;
            }
            if let Some(value) = extract_quoted_value(&contents, "subtitle =") {
                subtitle = strip_html_tags(&value);
            }
        }
    }

    HeaderData { title, subtitle }
}

fn resolve_header_path() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    for _ in 0..6 {
        let candidate = dir.join("src/components/Header.astro");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn extract_quoted_value(contents: &str, marker: &str) -> Option<String> {
    let index = contents.find(marker)?;
    let after_marker = &contents[index + marker.len()..];
    let quote_pos = after_marker.find(['"', '\''])?;
    let quote_char = after_marker.chars().nth(quote_pos)?;
    let rest = &after_marker[quote_pos + 1..];
    let end = rest.find(quote_char)?;
    Some(rest[..end].trim().to_string())
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn load_posts(dir: &Path, base_url: &str, published_only: bool) -> io::Result<Vec<Post>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut posts = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let (frontmatter, body) = split_frontmatter(&content);

        if published_only {
            if let Some(published) = frontmatter.get("published") {
                if published.to_lowercase() != "true" {
                    continue;
                }
            }
        }

        let slug = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("post");

        let date = frontmatter
            .get("created_at")
            .cloned()
            .or_else(|| date_from_slug(slug));

        let title = frontmatter
            .get("title")
            .cloned()
            .unwrap_or_else(|| title_from_slug(slug));

        let description = frontmatter.get("description").cloned().unwrap_or_default();
        let body_text = body.trim();
        let body = if body_text.is_empty() && !description.is_empty() {
            description
        } else {
            body_text.to_string()
        };

        let sort_key = date.clone().unwrap_or_else(|| slug.to_string());
        let url = format!("{}/{}", base_url.trim_end_matches('/'), slug);

        posts.push(Post {
            title,
            date: date.unwrap_or_default(),
            body,
            url,
            sort_key,
        });
    }

    posts.sort_by(|a, b| b.sort_key.cmp(&a.sort_key));
    Ok(posts)
}

fn split_frontmatter(contents: &str) -> (HashMap<String, String>, &str) {
    let mut map = HashMap::new();
    if !contents.starts_with("---") {
        return (map, contents);
    }

    let mut offset = 0usize;
    let mut lines = contents.lines();
    let first = lines.next();
    if first != Some("---") {
        return (map, contents);
    }
    offset += first.unwrap_or("").len() + 1;

    for line in lines {
        if line == "---" {
            offset += line.len() + 1;
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            map.insert(
                key.trim().to_string(),
                clean_frontmatter_value(value.trim()),
            );
        }
        offset += line.len() + 1;
    }

    let body = contents.get(offset..).unwrap_or("");
    (map, body)
}

fn clean_frontmatter_value(value: &str) -> String {
    let trimmed = value.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

fn load_gallery(dir: &Path) -> io::Result<Vec<Post>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut posts = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let is_image = matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp");
        if !is_image {
            continue;
        }

        let filename = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("image");

        let sort_key = match entry.metadata().and_then(|meta| meta.modified()) {
            Ok(modified) => match modified.duration_since(std::time::UNIX_EPOCH) {
                Ok(duration) => format!("{:020}", duration.as_secs()),
                Err(_) => filename.to_string(),
            },
            Err(_) => filename.to_string(),
        };

        posts.push(Post {
            title: filename.to_string(),
            date: String::new(),
            body: format!("Image file: {}", path.display()),
            url: path.to_string_lossy().to_string(),
            sort_key,
        });
    }

    posts.sort_by(|a, b| b.sort_key.cmp(&a.sort_key));
    Ok(posts)
}

fn title_from_slug(slug: &str) -> String {
    slug.replace(['-', '_'], " ")
        .split_whitespace()
        .map(capitalize)
        .collect::<Vec<String>>()
        .join(" ")
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn date_from_slug(slug: &str) -> Option<String> {
    if slug.len() >= 10
        && slug
            .chars()
            .all(|c| c.is_ascii_digit() || c == '_' || c == '-')
    {
        let normalized = slug.replace('_', "-");
        return Some(normalized);
    }
    None
}

fn wrap_markdown(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let width = width.max(10);
    for raw in text.lines() {
        if raw.trim().is_empty() {
            lines.push(String::new());
            continue;
        }

        let trimmed = raw.trim_end();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let prefix = &trimmed[..2];
            let content = trimmed[2..].trim();
            lines.extend(wrap_line(content, width, prefix));
        } else {
            lines.extend(wrap_line(trimmed, width, ""));
        }
    }
    lines
}

fn wrap_line(text: &str, width: usize, prefix: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let indent = " ".repeat(prefix.len());
    let mut current = String::new();
    let mut first = true;

    for word in text.split_whitespace() {
        let prefix_now = if first { prefix } else { &indent };
        if current.is_empty() {
            current = format!("{}{}", prefix_now, word);
            first = false;
            continue;
        }

        if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = format!("{}{}", prefix_now, word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(prefix.to_string());
    }

    lines
}

fn clamp_text(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        return text.to_string();
    }
    let mut clipped = text
        .chars()
        .take(max_width.saturating_sub(1))
        .collect::<String>();
    clipped.push('…');
    clipped
}

fn format_date(date: &str) -> String {
    if date.len() >= 10 {
        date.chars().take(10).collect()
    } else {
        date.to_string()
    }
}

fn open_url(url: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map(|_| ())
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn().map(|_| ())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "open not supported on this OS",
        ))
    }
}

fn print_help() {
    println!("johnjeong - terminal edition");
    println!();
    println!("Usage:");
    println!("  johnjeong");
    println!("  johnjeong --help");
    println!("  johnjeong --version");
    println!();
    println!("Keys:");
    println!("  1-6    switch tabs");
    println!("  g      gallery tab");
    println!("  ↑/↓    move selection");
    println!("  pgup/dn scroll content");
    println!("  o/enter open link");
    println!("  q      quit");
    println!();
    println!("Content:");
    println!("  Set JOHNJEONG_CONTENT_DIR to a part-of-my-brain directory.");
}
