import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Channel, Cluster, Message, Settings, Status } from "./types";

export const api = {
  channels: () => invoke<Channel[]>("get_channels"),
  clusters: (topic: string) => invoke<Cluster[]>("get_clusters", { topic }),
  messages: (clusterId: string) => invoke<Message[]>("get_messages", { clusterId }),
  markRead: (id: string) => invoke("mark_read", { id }),
  markClusterRead: (clusterId: string) => invoke("mark_cluster_read", { clusterId }),
  markChannelRead: (topic: string) => invoke("mark_channel_read", { topic }),
  deleteMessage: (id: string) => invoke("delete_message", { id }),
  deleteCluster: (clusterId: string) => invoke("delete_cluster", { clusterId }),
  deleteChannel: (topic: string) => invoke("delete_channel", { topic }),
  getSettings: () => invoke<Settings>("get_settings"),
  getStatus: () => invoke<Status>("get_status"),
  saveSettings: (serverUrl: string, token: string, topics: string[]) =>
    invoke("save_settings", { serverUrl, token, topics }),
};

export function onInboxChanged(cb: () => void): Promise<UnlistenFn> {
  return listen("inbox-changed", cb);
}

export function onStatus(cb: (s: Status) => void): Promise<UnlistenFn> {
  return listen<Status>("status", (e) => cb(e.payload));
}
