import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { streamChat, prettyModel, type Message } from "./lib/chat";
import "./App.css";

function App() {
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [vaultPath, setVaultPath] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  function refreshVault() {
    invoke<string | null>("get_vault_path").then((p) => setVaultPath(p ?? null));
  }

  useEffect(() => {
    invoke<boolean>("has_api_key").then((ok) => {
      setHasKey(ok);
      if (!ok) setShowSettings(true);
    });
    refreshVault();
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

    // Patch the last (assistant) message as tokens/sources stream in.
    const patchLast = (fn: (last: Message) => Message) =>
      setMessages((m) => {
        const copy = [...m];
        copy[copy.length - 1] = fn(copy[copy.length - 1]);
        return copy;
      });

    try {
      await streamChat(next, {
        onMeta: (m) => patchLast((last) => ({ ...last, meta: m })),
        onToken: (t) => patchLast((last) => ({ ...last, content: last.content + t })),
        onSources: (s) => patchLast((last) => ({ ...last, sources: s })),
        onError: (e) => setError(e),
      });
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
        onVaultChanged={refreshVault}
        onClose={
          hasKey
            ? () => {
                refreshVault();
                setShowSettings(false);
              }
            : undefined
        }
      />
    );
  }

  const vaultName = vaultPath?.split("/").filter(Boolean).pop();

  return (
    <main className="amber">
      <header className="topbar">
        <span className="brand">amber</span>
        <button
          className={`vault-status ${vaultPath ? "on" : "off"}`}
          onClick={() => setShowSettings(true)}
          title={vaultPath ?? "No vault connected — click to set one"}
        >
          <span className="dot" />
          {vaultPath ? `vault: ${vaultName}` : "no vault"}
        </button>
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
            {m.role === "assistant" && m.meta && (
              <div className="model-label" title="task-routed model (M4)">
                ✦ {prettyModel(m.meta.model)} · {m.meta.mode}
              </div>
            )}
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
  onVaultChanged,
  onClose,
}: {
  hasKey: boolean;
  onChanged: (hasKey: boolean) => void;
  onVaultChanged: () => void;
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

  async function saveVault(path = vault) {
    setBusy(true);
    setErr(null);
    try {
      await invoke("set_vault_path", { path });
      setVault(path);
      onVaultChanged();
      setNote(path.trim() ? "Vault connected." : "Vault disconnected.");
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function browse() {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") {
      await saveVault(picked); // chosen folder is valid — connect immediately
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

        <label className="field-label">Knowledge vault (folder)</label>
        <div className="field-row">
          <input
            type="text"
            value={vault}
            onChange={(e) => setVault(e.currentTarget.value)}
            placeholder="/Users/tucker/FOUNDRY"
            onKeyDown={(e) => e.key === "Enter" && saveVault()}
          />
          <button className="secondary browse" onClick={browse} disabled={busy}>
            Browse…
          </button>
        </div>
        <button className="secondary" onClick={() => saveVault()} disabled={busy}>
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
