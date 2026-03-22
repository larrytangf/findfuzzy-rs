use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};

/// A terminal tool combining fd (file discovery) and fzf (fuzzy search)
#[derive(Parser)]
#[command(name = "findfuzzy")]
#[command(about = "Fast file finder with fuzzy search", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Find files by pattern
    Find {
        /// Search pattern (supports regex)
        #[arg(value_name = "PATTERN")]
        pattern: Option<String>,

        /// Directory to search in
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Search only files (not directories)
        #[arg(short = 'f', long)]
        files_only: bool,

        /// Search only directories
        #[arg(short = 'd', long)]
        dirs_only: bool,

        /// Case-insensitive search
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Maximum depth
        #[arg(short = 'D', long)]
        max_depth: Option<usize>,
    },

    /// Fuzzy search through input
    Fzf {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: Option<String>,

        /// Case-insensitive search
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Show only top N results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },

    /// Combined: find files and fuzzy search
    Search {
        /// Search pattern
        #[arg(value_name = "PATTERN")]
        pattern: Option<String>,

        /// Directory to search in
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Case-insensitive search
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Maximum depth
        #[arg(short = 'D', long)]
        max_depth: Option<usize>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Find {
            pattern,
            path,
            files_only,
            dirs_only,
            ignore_case,
            max_depth,
        } => {
            find_files(
                pattern.as_deref(),
                &path,
                files_only,
                dirs_only,
                ignore_case,
                max_depth,
            );
        }
        Commands::Fzf {
            query,
            ignore_case,
            limit,
        } => {
            fuzzy_search(query.as_deref(), ignore_case, limit);
        }
        Commands::Search {
            pattern,
            path,
            ignore_case,
            max_depth,
        } => {
            let files = find_files_vec(
                pattern.as_deref(),
                &path,
                false,
                false,
                ignore_case,
                max_depth,
            );
            if !files.is_empty() {
                println!("Found {} files. Fuzzy searching...\n", files.len());
                fuzzy_search_vec(&files, None, ignore_case, files.len());
            }
        }
    }
}

fn find_files(
    pattern: Option<&str>,
    path: &str,
    files_only: bool,
    dirs_only: bool,
    ignore_case: bool,
    max_depth: Option<usize>,
) {
    let results = find_files_vec(pattern, path, files_only, dirs_only, ignore_case, max_depth);
    for result in results {
        println!("{}", result);
    }
}

fn find_files_vec(
    pattern: Option<&str>,
    path: &str,
    files_only: bool,
    dirs_only: bool,
    ignore_case: bool,
    max_depth: Option<usize>,
) -> Vec<String> {
    let mut results = Vec::new();
    let path_buf = PathBuf::from(path);

    if path_buf.exists() {
        walk_directory(
            &path_buf,
            pattern,
            files_only,
            dirs_only,
            ignore_case,
            max_depth,
            0,
            &mut results,
        );
    } else {
        eprintln!("Error: Path '{}' does not exist", path);
    }

    results.sort();
    results
}

fn walk_directory(
    dir: &Path,
    pattern: Option<&str>,
    files_only: bool,
    dirs_only: bool,
    ignore_case: bool,
    max_depth: Option<usize>,
    current_depth: usize,
    results: &mut Vec<String>,
) {
    if let Some(max) = max_depth {
        if current_depth > max {
            return;
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let path = entry.path();
                let is_dir = metadata.is_dir();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                let matches = if let Some(pat) = pattern {
                    if ignore_case {
                        file_name.to_lowercase().contains(&pat.to_lowercase())
                    } else {
                        file_name.contains(pat)
                    }
                } else {
                    true
                };

                if matches {
                    let should_include = if is_dir {
                        !files_only
                    } else {
                        !dirs_only
                    };

                    if should_include {
                        if let Some(path_str) = path.to_str() {
                            results.push(path_str.to_string());
                        }
                    }
                }

                if is_dir {
                    walk_directory(
                        &path,
                        pattern,
                        files_only,
                        dirs_only,
                        ignore_case,
                        max_depth,
                        current_depth + 1,
                        results,
                    );
                }
            }
        }
    }
}

fn fuzzy_search(query: Option<&str>, ignore_case: bool, limit: usize) {
    let stdin = io::stdin();
    let mut lines = Vec::new();

    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            lines.push(line);
        }
    }

    fuzzy_search_vec(&lines, query, ignore_case, limit);
}

fn fuzzy_search_vec(
    items: &[String],
    query: Option<&str>,
    ignore_case: bool,
    limit: usize,
) {
    let query = query.unwrap_or("");

    let mut scored_items: Vec<(String, f32)> = items
        .iter()
        .filter_map(|item| {
            let score = calculate_fuzzy_score(item, query, ignore_case);
            if score > 0.0 {
                Some((item.clone(), score))
            } else {
                None
            }
        })
        .collect();

    scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (item, _score) in scored_items.iter().take(limit) {
        println!("{}", item);
    }
}

fn calculate_fuzzy_score(item: &str, query: &str, ignore_case: bool) -> f32 {
    if query.is_empty() {
        return 1.0;
    }

    let item_cmp = if ignore_case {
        item.to_lowercase()
    } else {
        item.to_string()
    };
    let query_cmp = if ignore_case {
        query.to_lowercase()
    } else {
        query.to_string()
    };

    if item_cmp.contains(&query_cmp) {
        // Exact substring match
        return 2.0;
    }

    let mut query_chars = query_cmp.chars().peekable();
    let mut item_chars = item_cmp.chars().peekable();
    let mut matched = 0;
    let mut total_chars = 0;

    while let Some(&qc) = query_chars.peek() {
        let mut found = false;
        while let Some(&ic) = item_chars.peek() {
            total_chars += 1;
            item_chars.next();
            if ic == qc {
                matched += 1;
                query_chars.next();
                found = true;
                break;
            }
        }
        if !found {
            return 0.0;
        }
    }

    if matched == 0 {
        return 0.0;
    }

    let score = matched as f32 / total_chars as f32;
    score
}