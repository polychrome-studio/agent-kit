// Shared chat-streaming plumbing for both surfaces — the main window's
// conversation (App.tsx) and the floating command bar (CommandBar.tsx).
// Both call the same `chat` Tauri command; this wraps the Channel handshake so
// neither has to re-derive the StreamEvent protocol.

import { Channel, invoke } from "@tauri-apps/api/core";

export type Role = "user" | "assistant" | "system";

export interface Meta {
  mode: string;
  model: string;
}

export interface Message {
  role: Role;
  content: string;
  sources?: string[];
  meta?: Meta;
}

// Mirrors the Rust StreamEvent enum (serde lowercase tag/content).
export type StreamEvent =
  | { type: "meta"; data: Meta }
  | { type: "sources"; data: string[] }
  | { type: "token"; data: string }
  | { type: "done" }
  | { type: "error"; data: string };

export interface StreamHandlers {
  onMeta?: (meta: Meta) => void;
  onSources?: (sources: string[]) => void;
  onToken?: (token: string) => void;
  onError?: (message: string) => void;
  onDone?: () => void;
}

/** "anthropic/claude-opus-4.8:online" → "opus 4.8" for a compact UI label. */
export function prettyModel(model: string): string {
  return model
    .split("/")
    .pop()!
    .replace(/:online$/, "")
    .replace(/^claude-/, "")
    .replace(/-/g, " ");
}

/** Stream a chat completion. Tokens arrive on `handlers.onToken` as they generate. */
export async function streamChat(messages: Message[], handlers: StreamHandlers): Promise<void> {
  const onEvent = new Channel<StreamEvent>();
  onEvent.onmessage = (ev) => {
    if (ev.type === "token") handlers.onToken?.(ev.data);
    else if (ev.type === "meta") handlers.onMeta?.(ev.data);
    else if (ev.type === "sources") handlers.onSources?.(ev.data);
    else if (ev.type === "error") handlers.onError?.(ev.data);
    else if (ev.type === "done") handlers.onDone?.();
  };
  await invoke("chat", { messages, onEvent });
}
