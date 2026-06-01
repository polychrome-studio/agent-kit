// Amber — vault retrieval (M2).
// "The filesystem is the memory." Read an index file + keyword-grep the markdown
// vault, return the most relevant notes to ground the model. No vector DB (yet).

use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILES_SCANNED: usize = 4000;
const SCORE_READ_CAP: usize = 32 * 1024; // bytes read per file while scoring
const NOTE_INCLUDE_CAP: usize = 4000; // chars of each note injected as context
const INDEX_INCLUDE_CAP: usize = 6000; // chars of the index injected
const TOP_NOTES: usize = 3;

// Index file candidates relative to the vault root, in priority order.
// First match wins — gives the model the "map" cheaply before drilling into notes.
const INDEX_CANDIDATES: &[&str] = &[
    "knowledge/_index/master.md",
    "index.md",
    "INDEX.md",
    "_index.md",
    "README.md",
    "Home.md",
];

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "are", "but", "not", "you", "your", "with", "what", "who",
    "how", "why", "when", "where", "does", "did", "was", "were", "this", "that",
    "they", "them", "from", "have", "has", "had", "can", "could", "would", "should",
    "about", "into", "than", "then", "there", "tell", "give", "show", "find", "know",
    "get", "got", "let", "make", "want", "need", "amber",
];

pub struct Note {
    pub path: String, // vault-relative
    pub content: String,
}

pub struct VaultContext {
    pub index: Option<String>,
    pub notes: Vec<Note>,
}

impl VaultContext {
    fn is_empty(&self) -> bool {
        self.index.is_none() && self.notes.is_empty()
    }
}

/// Truncate a String to at most `max` bytes, respecting UTF-8 char boundaries.
fn truncate_chars(s: &mut String, max: usize) {
    if s.len() <= max {
        return;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
}

fn keywords(query: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in query.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        let w = raw.trim();
        if w.len() >= 3 && !STOPWORDS.contains(&w) && !out.iter().any(|e| e == w) {
            out.push(w.to_string());
        }
    }
    out
}

fn collect_markdown(root: &Path, out: &mut Vec<PathBuf>) {
    if out.len() >= MAX_FILES_SCANNED {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_FILES_SCANNED {
            return;
        }
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue; // skip hidden dirs/files (.obsidian, .git, .DS_Store, …)
        }
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

fn read_capped(path: &Path, cap: usize) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let slice = if bytes.len() > cap { &bytes[..cap] } else { &bytes[..] };
    Some(String::from_utf8_lossy(slice).into_owned())
}

fn read_index(vault: &Path) -> Option<String> {
    for rel in INDEX_CANDIDATES {
        let p = vault.join(rel);
        if p.is_file() {
            let mut s = read_capped(&p, INDEX_INCLUDE_CAP * 2)?;
            truncate_chars(&mut s, INDEX_INCLUDE_CAP);
            return Some(s);
        }
    }
    None
}

/// Read the index, score every markdown note against the query, return the top few.
pub fn build_context(vault: &Path, query: &str) -> VaultContext {
    let index = read_index(vault);
    let kws = keywords(query);
    if kws.is_empty() {
        return VaultContext { index, notes: vec![] };
    }

    let mut files = Vec::new();
    collect_markdown(vault, &mut files);

    let mut scored: Vec<(i64, PathBuf)> = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(vault)
            .unwrap_or(path)
            .to_string_lossy()
            .to_lowercase();
        let Some(content) = read_capped(path, SCORE_READ_CAP) else {
            continue;
        };
        let lc = content.to_lowercase();
        let mut score: i64 = 0;
        for kw in &kws {
            score += lc.matches(kw.as_str()).count() as i64;
            if rel.contains(kw.as_str()) {
                score += 8; // filename/path match is a strong relevance signal
            }
        }
        if score == 0 {
            continue;
        }
        // Prefer the distilled knowledge layer over raw transcripts/journal entries.
        if rel.contains("/wiki/") || rel.starts_with("wiki/") {
            score += 5;
        } else if rel.contains("knowledge/") {
            score += 2;
        }
        scored.push((score, path.clone()));
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(TOP_NOTES);

    let notes = scored
        .into_iter()
        .filter_map(|(_, path)| {
            let rel = path
                .strip_prefix(vault)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            let mut content = read_capped(&path, NOTE_INCLUDE_CAP * 2)?;
            if content.len() > NOTE_INCLUDE_CAP {
                truncate_chars(&mut content, NOTE_INCLUDE_CAP);
                content.push_str("\n…(truncated)");
            }
            Some(Note { path: rel, content })
        })
        .collect();

    VaultContext { index, notes }
}

/// Tool (`search_vault`): score the vault against a query and return a readable digest
/// of the top matches — each note's path + a snippet — plus the list of paths (for the
/// Sources chips). The agent uses the digest to decide which notes to `read_note` in full.
pub fn search_digest(vault: &Path, query: &str) -> (String, Vec<String>) {
    let ctx = build_context(vault, query);
    if ctx.notes.is_empty() {
        return ("No matching notes found in the vault.".into(), vec![]);
    }
    let mut out = String::from("Top matching notes (use read_note to open one in full):\n");
    let mut paths = Vec::new();
    for n in &ctx.notes {
        paths.push(n.path.clone());
        // A short snippet is enough to decide relevance; read_note fetches the rest.
        let mut snippet = n.content.clone();
        truncate_chars(&mut snippet, 500);
        out.push_str(&format!("\n### {}\n{snippet}\n", n.path));
    }
    (out, paths)
}

/// Tool (`read_note`): read one note in full by vault-relative path. Refuses paths that
/// escape the vault root (the vault is read-only and sandboxed).
pub fn read_note(vault: &Path, rel: &str) -> Result<String, String> {
    let root = vault
        .canonicalize()
        .map_err(|e| format!("vault unavailable: {e}"))?;
    let target = root.join(rel.trim_start_matches('/'));
    let canon = target
        .canonicalize()
        .map_err(|_| format!("no such note: {rel}"))?;
    if !canon.starts_with(&root) {
        return Err("refused: path is outside the vault".into());
    }
    let mut s = read_capped(&canon, NOTE_INCLUDE_CAP * 4)
        .ok_or_else(|| format!("could not read: {rel}"))?;
    if s.len() >= NOTE_INCLUDE_CAP * 4 {
        truncate_chars(&mut s, NOTE_INCLUDE_CAP * 4);
        s.push_str("\n…(truncated)");
    }
    Ok(s)
}

/// Format the vault context (index + notes) to append after a persona preamble.
/// Returns None when there's nothing to inject. The persona/voice lives in
/// `router::Mode::persona` now (M4) — this is just the knowledge block.
#[allow(dead_code)]
pub fn context_block(ctx: &VaultContext) -> Option<String> {
    if ctx.is_empty() {
        return None;
    }
    let mut s = String::new();
    if let Some(index) = &ctx.index {
        s.push_str("# What you remember about Tucker's world (the map)\n");
        s.push_str(index);
        s.push_str("\n\n");
    }
    if !ctx.notes.is_empty() {
        s.push_str("# Relevant to what he just asked\n");
        for n in &ctx.notes {
            s.push_str("\n## ");
            s.push_str(&n.path);
            s.push('\n');
            s.push_str(&n.content);
            s.push('\n');
        }
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Smoke test against the real FOUNDRY vault. Skips cleanly if it's absent
    // (CI, another machine) so it never fails spuriously.
    #[test]
    fn retrieves_real_foundry_notes() {
        let vault = Path::new("/Users/tucker/FOUNDRY");
        if !vault.is_dir() {
            eprintln!("FOUNDRY not present — skipping retrieval smoke test");
            return;
        }
        let ctx = build_context(vault, "what is the praxis platform");
        let paths: Vec<&str> = ctx.notes.iter().map(|n| n.path.as_str()).collect();
        eprintln!(
            "index: {} bytes | notes: {:?}",
            ctx.index.as_ref().map_or(0, |s| s.len()),
            paths
        );
        assert!(ctx.index.is_some(), "should find the master index file");
        assert!(!ctx.notes.is_empty(), "should retrieve at least one note");
        assert!(
            paths.iter().any(|p| p.contains("praxis")),
            "the praxis note should rank in the top notes, got: {paths:?}"
        );
    }
}
