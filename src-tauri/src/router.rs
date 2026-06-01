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
    /// The OpenRouter slug that answers in this mode.
    pub fn model(&self) -> &'static str {
        match self {
            Mode::Quick => "anthropic/claude-haiku-4.5",
            Mode::Companion => "anthropic/claude-sonnet-4.6",
            Mode::Research => "anthropic/claude-opus-4.8",
        }
    }

    /// Short label surfaced in the UI (next to the model name) for trust + cost awareness.
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Quick => "quick",
            Mode::Companion => "companion",
            Mode::Research => "research",
        }
    }

    /// Quick tasks don't need the vault; companion + research do (recall / grounding).
    pub fn uses_vault(&self) -> bool {
        !matches!(self, Mode::Quick)
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
                 his files. The notes below (if any) are YOUR MEMORY of Tucker and his world: \
                 things he's written, decided, and been working on. Treat them as what you \
                 already know about him, not documents to quote. Never say \"according to your \
                 vault,\" never cite filenames, never announce that you looked something up — \
                 you just know it. Voice: a sharp friend who's been paying attention. Warm, \
                 direct, concise — sentences, not paragraphs. No preamble; lead with the answer. \
                 Weave in what he already knows or has in flight, anticipate his next move, and \
                 offer to go deeper instead of dumping everything. If your memory doesn't cover \
                 it, just answer naturally."
            }
            Mode::Research => {
                "You are Amber, doing research for Tucker against his knowledge vault. The notes \
                 below are his knowledge base. Be pragmatic, precise, and well-structured — lead \
                 with the answer, then the support. When a specific claim rests on a particular \
                 note, you may name that note so he can verify it. Clearly distinguish what's \
                 grounded in his notes from what's general knowledge. This is work, not \
                 conversation — no companion chit-chat, no filler."
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
        assert!(!Mode::Quick.uses_vault());
        assert!(Mode::Companion.uses_vault());
        assert!(Mode::Research.show_sources());
        assert!(!Mode::Companion.show_sources());
    }
}
