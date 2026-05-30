import { useEffect, useRef, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import "./App.css";

type Role = "user" | "assistant" | "system";
interface Message {
  role: Role;
  content: string;
  sources?: string[];
}

// Mirrors the Rust StreamEvent enum (serde lowercase tag/content).
type StreamEvent =
  | { type: "sources"; data: string[] }
  | { type: "token"; data: string }
  | { type: "done" }
  | { type: "error"; data: string };

function App() {
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<boolean>("has_api_key").then((ok) => {
      setHasKey(ok);
      if (!ok) setShowSettings(true);
    });
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
      setMessages((m) => {
        const copy = [...m];
        const last = copy[copy.length - 1];
        if (ev.type === "token") {
          copy[copy.length - 1] = { ...last, content: last.content + ev.data };
        } else if (ev.type === "sources") {
          copy[copy.length - 1] = { ...last, sources: ev.data };
        }
        return copy;
      });
      if (ev.type === "error") setError(ev.data);
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

  if (showSettings) {
    return (
      <Settings
        hasKey={hasKey}
        onChanged={(ok) => setHasKey(ok)}
        onClose={hasKey ? () => setShowSettings(false) : undefined}
      />
    );
  }

  return (
    <main className="amber">
      <header className="topbar">
        <span className="brand">amber</span>
        <button className="ghost" onClick={() => setShowSettings(true)}>
          settings
        </button>
      </header>

      <div className="thread" ref={scrollRef}>
        {messages.length === 0 && (
          <div className="empty">Ask Amber anything — grounded in your vault.</div>
        )}
        {messages.map((m, i) => (
          <div key={i} className={`msg ${m.role}`}>
            <div className="bubble">
              {m.content || (streaming && i === messages.length - 1 ? "▍" : "")}
            </div>
            {m.sources && m.sources.length > 0 && (
              <div className="sources">
                <span className="sources-label">grounded in</span>
                {m.sources.map((s) => (
                  <span key={s} className="chip">{s}</span>
                ))}
              </div>
            )}
          </div>
        ))}
        {error && (
          <div className="msg error">
            <div className="bubble">{error}</div>
          </div>
        )}
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

function Settings({
  hasKey,
  onChanged,
  onClose,
}: {
  hasKey: boolean;
  onChanged: (hasKey: boolean) => void;
  onClose?: () => void;
}) {
  const [key, setKey] = useState("");
  const [vault, setVault] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);

  useEffect(() => {
    invoke<string | null>("get_vault_path").then((p) => p && setVault(p));
  }, []);

  async function saveKey() {
    if (!key.trim()) return;
    setBusy(true);
    setErr(null);
    try {
      await invoke("set_api_key", { key });
      setKey("");
      onChanged(true);
      setNote("API key saved.");
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveVault() {
    setBusy(true);
    setErr(null);
    try {
      await invoke("set_vault_path", { path: vault });
      setNote(vault.trim() ? "Vault connected." : "Vault disconnected.");
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="amber settings">
      <div className="card">
        <h1>amber</h1>
        <p className="sub">
          {hasKey
            ? "Settings — your key and vault."
            : "Paste your OpenRouter API key to begin."}
        </p>

        <label className="field-label">
          OpenRouter API key{" "}
          {hasKey && <span className="ok">● stored</span>}
        </label>
        <input
          type="password"
          value={key}
          autoFocus={!hasKey}
          onChange={(e) => setKey(e.currentTarget.value)}
          placeholder={hasKey ? "Paste a new key to replace…" : "sk-or-v1-…"}
          onKeyDown={(e) => e.key === "Enter" && saveKey()}
        />
        <button onClick={saveKey} disabled={busy || !key.trim()}>
          {hasKey ? "Replace key" : "Save key"}
        </button>

        <div className="divider" />

        <label className="field-label">Knowledge vault (folder path)</label>
        <input
          type="text"
          value={vault}
          onChange={(e) => setVault(e.currentTarget.value)}
          placeholder="/Users/tucker/FOUNDRY"
          onKeyDown={(e) => e.key === "Enter" && saveVault()}
        />
        <button className="secondary" onClick={saveVault} disabled={busy}>
          {vault.trim() ? "Connect vault" : "Disconnect vault"}
        </button>

        {err && <p className="errline">{err}</p>}
        {note && !err && <p className="noteline">{note}</p>}

        {onClose && (
          <button className="link" onClick={onClose}>
            ← back to chat
          </button>
        )}

        <p className="hint">
          Key is stored locally (app config, never the vault). Vault stays plain
          markdown you own — Amber only reads it. API key auth only, never a
          subscription token.
        </p>
      </div>
    </main>
  );
}

export default App;
