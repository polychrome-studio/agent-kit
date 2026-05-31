import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { streamChat } from "./lib/chat";

// The floating, borderless, always-on-top command bar (M3). Summoned by the
// global Option+Space shortcut (registered in Rust). Ephemeral one-shot grammar:
// every summon is a fresh query → streamed answer → Esc / blur dismisses.
export default function CommandBar() {
  const [query, setQuery] = useState("");
  const [answer, setAnswer] = useState("");
  const [sources, setSources] = useState<string[]>([]);
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const win = getCurrentWindow();

  function reset() {
    setQuery("");
    setAnswer("");
    setSources([]);
    setError(null);
    setStreaming(false);
  }

  useEffect(() => {
    inputRef.current?.focus();
    // Each re-summon (Rust emits this after show+focus) clears the last query.
    const unshow = win.listen("palette:show", () => {
      reset();
      setTimeout(() => inputRef.current?.focus(), 20);
    });
    // Raycast grammar: clicking away dismisses the bar.
    const unfocus = win.onFocusChanged(({ payload: focused }) => {
      if (!focused) win.hide();
    });
    return () => {
      unshow.then((f) => f());
      unfocus.then((f) => f());
    };
  }, []);

  async function run() {
    const text = query.trim();
    if (!text || streaming) return;
    setError(null);
    setAnswer("");
    setSources([]);
    setStreaming(true);
    try {
      await streamChat([{ role: "user", content: text }], {
        onToken: (t) => setAnswer((a) => a + t),
        onSources: (s) => setSources(s),
        onError: (e) => setError(e),
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setStreaming(false);
    }
  }

  const hasOutput = answer || streaming || error || sources.length > 0;

  return (
    <div
      className="palette"
      onKeyDown={(e) => {
        if (e.key === "Escape") win.hide();
      }}
    >
      <form
        className="palette-input"
        onSubmit={(e) => {
          e.preventDefault();
          run();
        }}
      >
        <span className="palette-glyph">✦</span>
        <input
          ref={inputRef}
          autoFocus
          value={query}
          disabled={streaming}
          onChange={(e) => setQuery(e.currentTarget.value)}
          placeholder={streaming ? "Amber is thinking…" : "Ask Amber…"}
        />
      </form>

      {hasOutput && (
        <div className="palette-body">
          {error ? (
            <div className="palette-error">{error}</div>
          ) : (
            <div className="palette-answer">
              {answer || (streaming ? "▍" : "")}
            </div>
          )}
          {sources.length > 0 && (
            <div className="sources">
              <span className="sources-label">grounded in</span>
              {sources.map((s) => (
                <span key={s} className="chip">
                  {s}
                </span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
