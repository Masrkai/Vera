// build.rs

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::io::Write;

/// Parses a simple `$key: #value;` token file.
/// An optional `// name: Display Name` line sets the human-readable theme name.
fn parse_scss(path: &PathBuf) -> (HashMap<String, String>, Option<String>) {
    let mut vars = HashMap::new();
    let mut display_name = None;
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("// name:") {
                let name = rest.trim();
                if !name.is_empty() {
                    display_name = Some(name.to_string());
                }
            } else if line.starts_with('$') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim().trim_start_matches('$').to_string();
                    let val = parts[1].trim().trim_end_matches(';').trim().to_string();
                    vars.insert(key, val);
                }
            }
        }
    }
    (vars, display_name)
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}

/// Turns a filename stem like `hot-pink` into `Hot Pink`.
fn title_case(stem: &str) -> String {
    stem.split(|c: char| c == '-' || c == '_' || c == ' ')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Emits a Rust expression that builds a `ThemeColors` from parsed SCSS vars.
fn theme_colors_literal(vars: &HashMap<String, String>) -> String {
    let color = |key: &str| {
        let hex = vars.get(key).map(|s| s.as_str()).unwrap_or("#000000");
        let (r, g, b) = hex_to_rgb(hex);
        format!("slint::Color::from_rgb_u8({r}, {g}, {b})")
    };
    format!(
        "ThemeColors {{ bg: {}, surface: {}, text: {}, muted: {}, border: {}, accent: {}, preview: {} }}",
        color("bg-color"),
        color("surface-color"),
        color("text-color"),
        color("text-muted"),
        color("border-color"),
        color("accent-color"),
        color("preview-bg"),
    )
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let ui_file_path = manifest_dir.join("ui").join("main.slint");
    let themes_dir = manifest_dir.join("themes");

    println!("cargo:rerun-if-changed={}", ui_file_path.display());
    // Watch the whole directory so dropping in a new theme file triggers a rebuild.
    println!("cargo:rerun-if-changed={}", themes_dir.display());

    // ── Discover themes ────────────────────────────────────────────────
    let mut theme_files: Vec<PathBuf> = fs::read_dir(&themes_dir)
        .expect("themes/ directory is missing")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().map(|ext| ext == "scss").unwrap_or(false))
        .collect();
    // Sort for a stable, deterministic theme order.
    theme_files.sort();

    let mut themes: Vec<(String, String, HashMap<String, String>)> = Vec::new();
    for path in &theme_files {
        println!("cargo:rerun-if-changed={}", path.display());
        let (vars, name) = parse_scss(path);
        let id = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let display = name.unwrap_or_else(|| title_case(&id));
        themes.push((id, display, vars));
    }

    assert!(
        !themes.is_empty(),
        "No theme files found in {}",
        themes_dir.display()
    );

    // Default theme is "dark" when present, otherwise the first theme.
    let default_index = themes
        .iter()
        .position(|(id, _, _)| id == "dark")
        .unwrap_or(0);

    let ids: Vec<String> = themes.iter().map(|(id, _, _)| format!("{id:?}")).collect();
    let names: Vec<String> = themes.iter().map(|(_, name, _)| format!("{name:?}")).collect();
    let fallback_colors = theme_colors_literal(&themes[default_index].2);

    // ── Generate Rust source ───────────────────────────────────────────
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("generated_themes.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    writeln!(f, "pub struct ThemeColors {{").unwrap();
    writeln!(f, "    pub bg: slint::Color,").unwrap();
    writeln!(f, "    pub surface: slint::Color,").unwrap();
    writeln!(f, "    pub text: slint::Color,").unwrap();
    writeln!(f, "    pub muted: slint::Color,").unwrap();
    writeln!(f, "    pub border: slint::Color,").unwrap();
    writeln!(f, "    pub accent: slint::Color,").unwrap();
    writeln!(f, "    pub preview: slint::Color,").unwrap();
    writeln!(f, "}}").unwrap();

    writeln!(f, "pub const THEME_IDS: &[&str] = &[{}];", ids.join(", ")).unwrap();
    writeln!(
        f,
        "pub const THEME_DISPLAY_NAMES: &[&str] = &[{}];",
        names.join(", ")
    )
    .unwrap();

    writeln!(f, "pub fn default_theme_index() -> usize {{ {default_index} }}").unwrap();

    writeln!(f, "pub fn get_theme(index: usize) -> ThemeColors {{").unwrap();
    writeln!(f, "    match index {{").unwrap();
    for (i, (_, _, vars)) in themes.iter().enumerate() {
        writeln!(f, "        {} => {},", i, theme_colors_literal(vars)).unwrap();
    }
    writeln!(f, "        _ => {fallback_colors},").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();

    slint_build::compile(ui_file_path).expect("Slint build failed");
}
