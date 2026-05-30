import { useEffect, useRef, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import "./App.css";

type Role = "user" | "assistant" | "system";
interface Message {
  role: Role;
  content: string;
}

// Mirrors the Rust StreamEvent enum (serde lowercase tag/content).
type StreamEvent =
  | { type: "token"; data: string }
  | { type: "done" }
  | { type: "error"; data: string };

function App() {
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<boolean>("has_api_key").then(setHasKey);
  }, []);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [messages]);

  async function send() {
    const text = input.trim();
    if (!text || streaming) return;
    setError(null);
    setInput("");

    const next: Message[] = [...messages, { role: "user", content: text }];
    setMessages([...next, { role: "assistant", content: "" }]);
    setStreaming(true);

    const onEvent = new Channel<StreamEvent>();
    onEvent.onmessage = (ev) => {
      if (ev.type === "token") {
        setMessages((m) => {
          const copy = [...m];
          copy[copy.length - 1] = {
            role: "assistant",
            content: copy[copy.length - 1].content + ev.data,
          };
          return copy;
        });
      } else if (ev.type === "error") {
        setError(ev.data);
      }
    };

    try {
      await invoke("chat", { messages: next, onEvent });
    } catch (e) {
      setError(String(e));
    } finally {
      setStreaming(false);
    }
  }

  if (hasKey === null) {
    return <main className="amber loading">…</main>;
  }

  if (!hasKey) {
    return <Settings onSaved={() => setHasKey(true)} />;
  }

  return (
    <main className="amber">
      <header className="topbar">
        <span className="brand">amber</span>
        <button className="ghost" onClick={() => setHasKey(false)}>
          settings
        </button>
      </header>

      <div className="thread" ref={scrollRef}>
        {messages.length === 0 && (
          <div className="empty">Ask Amber anything.</div>
        )}
        {messages.map((m, i) => (
          <div key={i} className={`msg ${m.role}`}>
            <div className="bubble">
              {m.content || (streaming && i === messages.length - 1 ? "▍" : "")}
            </div>
          </div>
        ))}
        {error && <div className="msg error"><div className="bubble">{error}</div></div>}
      </div>

      <form
        className="composer"
        onSubmit={(e) => {
          e.preventDefault();
          send();
        }}
      >
        <input
          autoFocus
          value={input}
          disabled={streaming}
          onChange={(e) => setInput(e.currentTarget.value)}
          placeholder={streaming ? "Amber is thinking…" : "Message Amber…"}
        />
        <button type="submit" disabled={streaming || !input.trim()}>
          ↑
        </button>
      </form>
    </main>
  );
}

function Settings({ onSaved }: { onSaved: () => void }) {
  const [key, setKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function save() {
    if (!key.trim()) return;
    setSaving(true);
    setErr(null);
    try {
      await invoke("set_api_key", { key });
      onSaved();
    } catch (e) {
      setErr(String(e));
    } finally {
      setSaving(false);
    }
  }

  return (
    <main className="amber settings">
      <div className="card">
        <h1>amber</h1>
        <p className="sub">
          Paste your <strong>OpenRouter API key</strong>. Stored in the macOS
          keychain — never written to disk or the vault.
        </p>
        <input
          type="password"
          value={key}
          autoFocus
          onChange={(e) => setKey(e.currentTarget.value)}
          placeholder="sk-or-v1-…"
          onKeyDown={(e) => e.key === "Enter" && save()}
        />
        {err && <p className="errline">{err}</p>}
        <button onClick={save} disabled={saving || !key.trim()}>
          {saving ? "Saving…" : "Save key"}
        </button>
        <p className="hint">
          Get one at openrouter.ai/keys — API key auth only, never a
          subscription token.
        </p>
      </div>
    </main>
  );
}

export default App;
