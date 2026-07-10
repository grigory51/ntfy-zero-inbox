<script lang="ts">
  import { onMount } from "svelte";
  import { api, onInboxChanged, onStatus } from "./lib/api";
  import type { Channel, Cluster, Message, Status } from "./lib/types";
  import { ago, cap, tier } from "./lib/time";
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
  let query = $state("");

  async function refresh() {
    if (view.name === "channels") channels = await api.channels();
    else if (view.name === "clusters") clusters = await api.clusters(view.topic);
    else if (view.name === "messages") messages = await api.messages(view.cluster.id);
  }

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

  const visibleChannels = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return channels;
    return channels.filter(
      (c) =>
        c.topic.toLowerCase().includes(q) ||
        (c.last_body ?? "").toLowerCase().includes(q) ||
        (c.last_title ?? "").toLowerCase().includes(q),
    );
  });

  const headerTitle = $derived(
    view.name === "clusters"
      ? "#" + view.topic
      : view.name === "messages"
        ? view.cluster.label || "Тема"
        : "Инбокс",
  );

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

  // Стабильный цвет аватара из имени.
  function hue(s: string): number {
    let h = 0;
    for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) >>> 0;
    return h % 360;
  }
  const initial = (s: string) => (s.trim()[0] ?? "#").toUpperCase();
</script>

<main class="shell">
  {#if view.name === "settings"}
    <Settings onclose={() => (view = { name: "channels" })} />
  {:else}
    <header class="top">
      <div class="titlebar">
        {#if view.name !== "channels"}
          <button class="ghost back" title="Назад" onclick={() => (view = { name: "channels" })}>‹</button>
        {/if}
        <span class="dot" class:on={status.connected}></span>
        <span class="title" title={headerTitle}>{headerTitle}</span>
        {#if view.name === "channels" && totalUnread > 0}
          <span class="badge">{cap(totalUnread)}</span>
        {/if}
        <span class="spacer"></span>
        {#if view.name === "clusters"}
          <button class="ghost" title="Прочитать канал" onclick={() => api.markChannelRead(view.topic)}>✓</button>
          <button class="ghost danger" title="Удалить канал" onclick={delChannel}>🗑</button>
        {:else if view.name === "messages"}
          <button class="ghost" title="Отметить тему обработанной" onclick={() => api.markClusterRead(view.cluster.id)}>✓</button>
          <button class="ghost danger" title="Удалить тему" onclick={delCluster}>🗑</button>
        {:else}
          <button class="ghost" title="Настройки" onclick={() => (view = { name: "settings" })}>⚙</button>
        {/if}
      </div>

      {#if view.name === "channels"}
        <input class="search" placeholder="Поиск по каналам…" bind:value={query} />
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
        {#if visibleChannels.length === 0}
          <div class="empty">{query ? "Ничего не найдено" : "Пусто. Открой ⚙ и добавь топики."}</div>
        {/if}
        {#each visibleChannels as ch (ch.topic)}
          <button class="item" class:unread={ch.unread > 0} onclick={() => (view = { name: "clusters", topic: ch.topic })}>
            <div class="avatar" style="background: hsl({hue(ch.topic)} 42% 42%)">{initial(ch.topic)}</div>
            <div class="item-body">
              <div class="item-title">#{ch.topic}</div>
              <div class="item-sub">{ch.last_title || ch.last_body || ""}</div>
            </div>
            <div class="item-meta">
              <span class="time">{ago(ch.last_time)}</span>
              {#if ch.unread > 0}
                <span class="badge">{cap(ch.unread)}</span>
              {:else}
                <span class="soft">{ch.cluster_count} тем</span>
              {/if}
            </div>
          </button>
        {/each}
      {:else if view.name === "clusters"}
        {#each clusters as cl (cl.id)}
          <button class="item" class:unread={cl.unread > 0} onclick={() => (view = { name: "messages", cluster: cl })}>
            <div class="avatar sq" style="background: hsl({hue(cl.id)} 40% 44%)">◆</div>
            <div class="item-body">
              <div class="item-title">{cl.label || cl.last_body}</div>
              <div class="item-sub">{cl.last_body}</div>
            </div>
            <div class="item-meta">
              <span class="time">{ago(cl.last_time)}</span>
              {#if cl.unread > 0}
                <span class="badge">{cap(cl.unread)}</span>
              {:else if cl.total > 1}
                <span class="soft">+{cl.total - 1}</span>
              {/if}
            </div>
          </button>
        {/each}
      {:else if view.name === "messages"}
        {#each messages as m (m.id)}
          <div class="msg tier-{tier(m.priority)}" class:unread={!m.read}>
            <div class="msg-body">
              {#if m.title}<div class="msg-title">{m.title}</div>{/if}
              <div class="msg-text">{m.body}</div>
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
  .shell {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
  }

  .top {
    border-bottom: 1px solid var(--border);
  }
  .titlebar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 10px 8px;
  }
  .back {
    font-size: 18px;
    padding: 0 4px;
  }
  .title {
    font-weight: 600;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 210px;
  }
  .spacer {
    flex: 1;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--warn);
    flex: none;
  }
  .dot.on {
    background: var(--ok);
  }

  .search {
    width: calc(100% - 20px);
    margin: 0 10px 10px;
    background: var(--surface);
    border: 1px solid transparent;
    border-radius: 8px;
    padding: 7px 10px;
    color: var(--text);
    font-size: 13px;
  }
  .search:focus {
    outline: none;
    border-color: var(--accent);
  }
  .search::placeholder {
    color: var(--muted);
  }

  .badge {
    background: var(--accent);
    color: #fff;
    border-radius: 999px;
    padding: 1px 7px;
    font-size: 11px;
    font-weight: 600;
    flex: none;
  }
  .soft {
    color: var(--muted);
    font-size: 11px;
  }

  .error {
    background: color-mix(in srgb, var(--err) 18%, transparent);
    color: var(--err);
    font-size: 12px;
    padding: 6px 12px;
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

  .list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 6px 8px;
  }
  .empty {
    color: var(--muted);
    text-align: center;
    padding: 44px 20px;
    font-size: 13px;
  }

  /* JetBrains-Toolbox-подобный ряд */
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 8px;
    background: none;
    border: none;
    border-radius: 8px;
    text-align: left;
    cursor: pointer;
  }
  .item:hover {
    background: var(--surface);
  }
  .avatar {
    width: 34px;
    height: 34px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    font-weight: 700;
    font-size: 15px;
    flex: none;
  }
  .avatar.sq {
    font-size: 13px;
    border-radius: 9px;
  }
  .item-body {
    flex: 1;
    min-width: 0;
  }
  .item-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .item.unread .item-title {
    color: var(--text);
    font-weight: 600;
  }
  .item-sub {
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 1px;
  }
  .item.unread .item-sub {
    color: var(--text);
  }
  .item-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    flex: none;
  }
  .time {
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }

  .msg {
    display: flex;
    gap: 8px;
    padding: 9px 10px;
    border-radius: 8px;
    border-left: 2px solid transparent;
  }
  .msg:hover {
    background: var(--surface);
  }
  .msg.unread {
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }
  .msg.unread .msg-title,
  .msg.unread .msg-text {
    font-weight: 600;
  }
  .msg.tier-high {
    border-left-color: var(--err);
  }
  .msg.tier-low {
    opacity: 0.72;
  }
  .msg-body {
    flex: 1;
    min-width: 0;
  }
  .msg-title {
    font-weight: 600;
    font-size: 13px;
  }
  .msg-text {
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
    background: var(--surface-hover);
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
    background: var(--surface-hover);
  }
  .icon-btn.danger {
    color: var(--err);
  }
  :global(button.ghost.danger:hover) {
    color: var(--err);
  }
</style>
