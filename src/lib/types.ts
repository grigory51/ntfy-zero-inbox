export interface Channel {
  topic: string;
  total: number;
  unread: number;
  cluster_count: number;
  last_title: string | null;
  last_body: string | null;
  last_time: number | null;
}

export interface Cluster {
  id: string;
  topic: string;
  label: string;
  total: number;
  unread: number;
  last_time: number;
  last_body: string;
}

export interface Message {
  id: string;
  topic: string;
  cluster_id: string;
  title: string | null;
  body: string;
  priority: number;
  tags: string[];
  time: number;
  read: boolean;
  click: string | null;
}

export interface Settings {
  server_url: string;
  token: string;
  topics: string[];
  since: string;
}

export interface Status {
  connected: boolean;
  error: string | null;
  model_ready: boolean;
}
