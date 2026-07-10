<script lang="ts">
  import { api } from "./api";
  import type { Settings } from "./types";

  let { onclose }: { onclose: () => void } = $props();

  let serverUrl = $state("");
  let token = $state("");
  let topicsRaw = $state("");
  let saving = $state(false);

  api.getSettings().then((s: Settings) => {
    serverUrl = s.server_url;
    token = s.token;
    topicsRaw = s.topics.join(", ");
  });

  async function save() {
    saving = true;
    const topics = topicsRaw
      .split(/[,\s]+/)
      .map((t) => t.trim())
      .filter(Boolean);
    await api.saveSettings(serverUrl, token, topics);
    saving = false;
    onclose();
  }
</script>

<div class="settings">
  <header>
    <button class="ghost" onclick={onclose}>‹ Назад</button>
    <strong>Настройки</strong>
    <span></span>
  </header>

  <label>
    Сервер
    <input bind:value={serverUrl} placeholder="https://ntfy.example.com" />
  </label>

  <label>
    Токен <span class="muted">(если сервер закрытый)</span>
    <input bind:value={token} type="password" placeholder="tk_…" />
  </label>

  <label>
    Топики <span class="muted">— через запятую, каждый = канал</span>
    <input bind:value={topicsRaw} placeholder="backups, ci, alerts" />
  </label>

  <button class="primary" onclick={save} disabled={saving}>
    {saving ? "Сохраняю…" : "Применить и переподключиться"}
  </button>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--muted);
  }
  input {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 10px;
    color: var(--text);
    font-size: 13px;
  }
  input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .muted {
    color: var(--muted);
    font-weight: 400;
  }
</style>
