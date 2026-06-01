// Amber — task router (M4). "Mode is the primitive."
//
// One classification per query drives three knobs at once: which model answers,
// which persona/voice Amber uses, and whether source chips are shown. This is the
// cost lever (cheap by default, frontier only when earned) and the tone selector
// in one place.
//
// Classification is HYBRID: instant heuristics decide the obvious cases (the common
// path stays snappy — no extra call), and a cheap model call breaks the tie only for
// genuinely ambiguous queries. See knowledge/wiki/roadmap.md (M4).

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
// Cheapest valid slug — classification is an easy task, keep the tie-breaker call tiny.
const CLASSIFIER_MODEL: &str = "anthropic/claude-3.5-haiku";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Trivial mechanical tasks (format, rewrite, translate). Cheap model, terse, no vault.
    Quick,
    /// The default — conversation, advice, recall, thinking together. Companion voice.
    Companion,
    /// Find / synthesize / explain from the vault. Frontier model, pragmatic, sources shown.
    Research,
}

impl Mode {
    /// The OpenRouter slug that answers in this mode. Web access is no longer a forced
    /// `:online` suffix — it's the model-decided `web_search` tool (see [[agent]]), so the
    /// model reaches the web only when it judges it needs to.
    pub fn model(&self) -> &'static str {
        match self {
            Mode::Quick => "anthropic/claude-haiku-4.5",
            Mode::Companion => "anthropic/claude-sonnet-4.6",
            Mode::Research => "anthropic/claude-opus-4.8",
        }
    }

    /// Whether the model gets the agent toolset (search_vault / read_note / web_search).
    /// Quick is mechanical (format/rewrite) — no tools, one shot, stays instant.
    pub fn tools(&self) -> bool {
        !matches!(self, Mode::Quick)
    }

    /// Short label surfaced in the UI (next to the model name) for trust + cost awareness.
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Quick => "quick",
            Mode::Companion => "companion",
            Mode::Research => "research",
        }
    }

    /// Only research mode surfaces source chips — Tucker doesn't want to see the
    /// sourcing during companion/quick work ("I don't need to see it's pulling from the vault").
    pub fn show_sources(&self) -> bool {
        matches!(self, Mode::Research)
    }

    /// The voice preamble. Always injected (even with no vault hit) so Amber's
    /// register is consistent. The vault context block, if any, is appended after this.
    pub fn persona(&self) -> &'static str {
        match self {
            Mode::Companion => {
                "You are Amber — Tucker's second brain and companion, not a search engine over \
                 his files. You have tools: `search_vault` and `read_note` to recall what he's \
                 written/decided/worked on, and `web_search` for anything current or external. \
                 Use them silently when a question turns on something specific about Tucker or the \
                 world — but don't over-search; for ordinary conversation just talk. Treat what you \
                 retrieve as YOUR MEMORY of him, not documents to quote: never say \"according to \
                 your vault,\" never cite filenames, never announce that you looked something up. \
                 Voice: a sharp friend who's been paying attention. Warm, direct, concise — \
                 sentences, not paragraphs. No preamble; lead with the answer. Weave in what he \
                 already knows or has in flight, anticipate his next move, offer to go deeper."
            }
            Mode::Research => {
                "You are Amber, doing research for Tucker. You have tools — USE THEM, don't answer \
                 from memory alone: `search_vault` then `read_note` to mine his own notes, and \
                 `web_search` for anything current, external, or missing from his notes. Don't just \
                 report that his notes are thin — GO FIND what's missing and bring it back. Search \
                 iteratively: search, read the promising hits, search again to fill gaps. Be \
                 pragmatic, precise, well-structured: lead with the answer, then the support. \
                 Clearly separate what came from his notes from what came from the web (cite the \
                 URL) from your own general knowledge. This is work — no companion chit-chat."
            }
            Mode::Quick => {
                "You are Amber. Do exactly the small task asked and nothing more. Answer in as \
                 few words as possible — just the result, no preamble, no explanation unless asked."
            }
        }
    }
}

/// Confident heuristic. Returns Some when the query clearly belongs to a mode; None
/// when it's ambiguous and worth a model call. Tuned so the common path (short
/// conversational queries → Companion) is instant and the model call is the exception.
pub fn heuristic(query: &str) -> Option<Mode> {
    let lc = query.to_lowercase();

    const QUICK: &[&str] = &[
        "format", "rewrite", "reword", "rephrase", "translate", "shorten", "capitalize",
        "tldr", "tl;dr", "proofread", "fix the grammar", "fix grammar", "spell check",
        "spellcheck", "lowercase", "uppercase",
    ];
    if QUICK.iter().any(|k| lc.contains(k)) {
        return Some(Mode::Quick);
    }

    const RESEARCH: &[&str] = &[
        "my notes", "my vault", "according to", "research ", "sources", "cite", "look up",
        "dig into", "compile", "across my", "what do my notes", "summarize the",
        "summarise the", "everything about", "deep dive",
    ];
    if RESEARCH.iter().any(|k| lc.contains(k)) {
        return Some(Mode::Research);
    }

    // Short and unsignalled → safely conversational. This is the hot path, kept instant.
    if lc.split_whitespace().count() <= 12 {
        return Some(Mode::Companion);
    }

    // Long and unsignalled → could be a research ask. Let the model break the tie.
    None
}

/// Classify a query into a mode. Heuristic first (instant), cheap model call only
/// when the heuristic abstains. Defaults to Companion if the model call fails.
pub async fn classify(query: &str, api_key: &str) -> Mode {
    if let Some(mode) = heuristic(query) {
        return mode;
    }
    classify_via_model(query, api_key).await.unwrap_or(Mode::Companion)
}

const CLASSIFY_PROMPT: &str = "You route a user's message to one mode. Reply with exactly one \
    word: quick, companion, or research.\n\
    - quick: trivial mechanical tasks (format, rewrite, translate, shorten, list, fix).\n\
    - research: find/synthesize/explain from the user's own notes; multi-part questions; \
    summarize across sources; \"what do my notes say about…\".\n\
    - companion: everything else — conversation, advice, recall, thinking together.";

async fn classify_via_model(query: &str, api_key: &str) -> Option<Mode> {
    let body = serde_json::json!({
        "model": CLASSIFIER_MODEL,
        "max_tokens": 5,
        "temperature": 0,
        "messages": [
            { "role": "system", "content": CLASSIFY_PROMPT },
            { "role": "user", "content": query },
        ],
    });
    let resp = reqwest::Client::new()
        .post(OPENROUTER_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("X-Title", "Amber")
        .json(&body)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().await.ok()?;
    let text = v["choices"][0]["message"]["content"].as_str()?.to_lowercase();
    if text.contains("research") {
        Some(Mode::Research)
    } else if text.contains("quick") {
        Some(Mode::Quick)
    } else if text.contains("companion") {
        Some(Mode::Companion)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristics_route_obvious_cases() {
        assert_eq!(heuristic("rewrite this paragraph to be tighter"), Some(Mode::Quick));
        assert_eq!(heuristic("what do my notes say about praxis"), Some(Mode::Research));
        assert_eq!(heuristic("research the vault for cost model decisions"), Some(Mode::Research));
        assert_eq!(heuristic("what should I focus on today"), Some(Mode::Companion));
        // Long + unsignalled → ambiguous, defer to the model.
        let long = "i was thinking about how the bridge product strategy connects to the broader \
                    platform play and whether the timing actually makes sense given everything else";
        assert_eq!(heuristic(long), None);
    }

    #[test]
    fn mode_knobs() {
        assert!(!Mode::Quick.tools());
        assert!(Mode::Companion.tools());
        assert!(Mode::Research.show_sources());
        assert!(!Mode::Companion.show_sources());
    }
}
