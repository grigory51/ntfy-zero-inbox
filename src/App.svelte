<script lang="ts">
  import { onMount } from "svelte";
  import { api, onInboxChanged, onStatus } from "./lib/api";
  import type { Channel, Cluster, Message, Status } from "./lib/types";
  import { ago } from "./lib/time";
  import Settings from "./lib/Settings.svelte";

  type View =
    | { name: "channels" }
    | { name: "clusters"; topic: string }
    | { name: "messages"; cluster: Cluster }
    | { name: "settings" };

  let view = $state<View>({ name: "channels" });
  let channels = $state<Channel[]>([]);
  let clusters = $state<Cluster[]>([]);
  let messages = $state<Message[]>([]);
  let status = $state<Status>({ connected: false, error: null, model_ready: false });

  async function refresh() {
    if (view.name === "channels") channels = await api.channels();
    else if (view.name === "clusters") clusters = await api.clusters(view.topic);
    else if (view.name === "messages") messages = await api.messages(view.cluster.id);
  }

  // Перезапрос при смене экрана.
  $effect(() => {
    void view;
    refresh();
  });

  onMount(() => {
    api.getStatus().then((s) => (status = s));
    const u1 = onInboxChanged(refresh);
    const u2 = onStatus((s) => (status = s));
    return () => {
      u1.then((f) => f());
      u2.then((f) => f());
    };
  });

  const totalUnread = $derived(channels.reduce((a, c) => a + c.unread, 0));

  async function delCluster() {
    if (view.name !== "messages") return;
    const topic = view.cluster.topic;
    await api.deleteCluster(view.cluster.id);
    view = { name: "clusters", topic };
  }

  async function delChannel() {
    if (view.name !== "clusters") return;
    await api.deleteChannel(view.topic);
    view = { name: "channels" };
  }
</script>

<main>
  {#if view.name === "settings"}
    <Settings onclose={() => (view = { name: "channels" })} />
  {:else}
    <header class="bar">
      {#if view.name === "channels"}
        <span class="dot" class:on={status.connected}></span>
        <strong>Инбокс</strong>
        {#if totalUnread > 0}<span class="badge">{totalUnread}</span>{/if}
        <span class="spacer"></span>
        <button class="ghost" title="Настройки" onclick={() => (view = { name: "settings" })}>⚙</button>
      {:else}
        <button class="ghost" onclick={() => (view = { name: "channels" })}>‹</button>
        <strong>
          #{view.name === "clusters" ? view.topic : view.cluster.topic}
        </strong>
        <span class="spacer"></span>
        {#if view.name === "clusters"}
          <button class="ghost" title="Прочитать всё" onclick={() => api.markChannelRead(view.topic)}>✓</button>
          <button class="ghost danger" title="Удалить канал" onclick={delChannel}>🗑</button>
        {:else if view.name === "messages"}
          <button class="ghost" title="Отметить тему обработанной" onclick={() => api.markClusterRead(view.cluster.id)}>✓</button>
          <button class="ghost danger" title="Удалить тему" onclick={delCluster}>🗑</button>
        {/if}
      {/if}
    </header>

    {#if status.error && !status.connected}
      <div class="error">{status.error}</div>
    {/if}
    {#if !status.model_ready}
      <div class="loader" title="Скачиваю модель эмбеддингов (один раз, ~23 МБ)">
        <div class="loader-track"><div class="loader-bar"></div></div>
        <span>Готовлю умную группировку…</span>
      </div>
    {/if}

    <div class="list">
      {#if view.name === "channels"}
        {#if channels.length === 0}
          <div class="empty">Пусто. Открой ⚙ и добавь топики.</div>
        {/if}
        {#each channels as ch (ch.topic)}
          <button class="row" onclick={() => (view = { name: "clusters", topic: ch.topic })}>
            <div class="row-main">
              <div class="row-title">
                <span class="chan">#{ch.topic}</span>
                {#if ch.unread > 0}<span class="badge sm">{ch.unread}</span>{/if}
              </div>
              <div class="row-sub">{ch.last_title || ch.last_body || ""}</div>
            </div>
            <div class="row-meta">
              <span>{ago(ch.last_time)}</span>
              <span class="muted">{ch.cluster_count} тем · {ch.total}</span>
            </div>
          </button>
        {/each}
      {:else if view.name === "clusters"}
        {#each clusters as cl (cl.id)}
          <button class="row" onclick={() => (view = { name: "messages", cluster: cl })}>
            <div class="row-main">
              <div class="row-title">
                <span class="label">{cl.label || cl.last_body}</span>
                {#if cl.unread > 0}<span class="badge sm">{cl.unread}</span>{/if}
              </div>
              <div class="row-sub">{cl.last_body}</div>
            </div>
            <div class="row-meta">
              <span>{ago(cl.last_time)}</span>
              {#if cl.total > 1}<span class="muted">+{cl.total - 1} ещё</span>{/if}
            </div>
          </button>
        {/each}
      {:else if view.name === "messages"}
        {#each messages as m (m.id)}
          <div class="msg" class:unread={!m.read}>
            <div class="msg-main">
              {#if m.title}<div class="msg-title">{m.title}</div>{/if}
              <div class="msg-body">{m.body}</div>
              <div class="msg-meta">
                <span>{ago(m.time)}</span>
                {#each m.tags as t}<span class="tag">{t}</span>{/each}
              </div>
            </div>
            <div class="msg-actions">
              {#if !m.read}
                <button class="icon-btn" title="Обработано" onclick={() => api.markRead(m.id)}>✓</button>
              {/if}
              <button class="icon-btn danger" title="Удалить" onclick={() => api.deleteMessage(m.id)}>🗑</button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</main>

<style>
  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }
  .spacer {
    flex: 1;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--warn);
  }
  .dot.on {
    background: var(--ok);
  }
  .badge {
    background: var(--accent);
    color: #fff;
    border-radius: 999px;
    padding: 1px 7px;
    font-size: 11px;
    font-weight: 600;
  }
  .badge.sm {
    padding: 0 6px;
    font-size: 10px;
  }
  .error {
    background: rgba(220, 38, 38, 0.15);
    color: var(--err);
    font-size: 12px;
    padding: 6px 12px;
  }
  .hint {
    color: var(--muted);
    font-size: 11px;
    padding: 4px 12px;
  }
  .list {
    flex: 1;
    overflow-y: auto;
  }
  .empty {
    color: var(--muted);
    text-align: center;
    padding: 40px 20px;
    font-size: 13px;
  }
  .row {
    width: 100%;
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 10px 12px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    text-align: left;
    cursor: pointer;
    color: var(--text);
  }
  .row:hover {
    background: var(--surface);
  }
  .row-main {
    flex: 1;
    min-width: 0;
  }
  .row-title {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .chan,
  .label {
    font-weight: 600;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row-sub {
    color: var(--muted);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }
  .row-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }
  .msg {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }
  .msg.unread {
    background: rgba(99, 102, 241, 0.08);
  }
  .msg-main {
    flex: 1;
    min-width: 0;
  }
  .msg-title {
    font-weight: 600;
    font-size: 13px;
  }
  .msg-body {
    font-size: 13px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .msg-meta {
    display: flex;
    gap: 6px;
    align-items: center;
    margin-top: 4px;
    font-size: 11px;
    color: var(--muted);
  }
  .tag {
    background: var(--surface);
    border-radius: 4px;
    padding: 0 5px;
  }
  .msg-actions {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .icon-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--ok);
    cursor: pointer;
    width: 24px;
    height: 24px;
    font-size: 12px;
  }
  .icon-btn:hover {
    background: var(--surface);
  }
  .icon-btn.danger {
    color: var(--err);
  }
  :global(button.ghost.danger:hover) {
    color: var(--err);
  }
  .loader {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }
  .loader-track {
    flex: 1;
    height: 3px;
    background: var(--surface);
    border-radius: 2px;
    overflow: hidden;
  }
  .loader-bar {
    height: 100%;
    width: 40%;
    background: var(--accent);
    border-radius: 2px;
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    0% {
      margin-left: -40%;
    }
    100% {
      margin-left: 100%;
    }
  }
</style>
