use crate::models::*;
use rusqlite::{params, Connection};

pub fn open(path: &std::path::Path) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS clusters (
            id        TEXT PRIMARY KEY,
            topic     TEXT NOT NULL,
            label     TEXT NOT NULL,
            centroid  BLOB NOT NULL,
            count     INTEGER NOT NULL,
            last_time INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS messages (
            id         TEXT PRIMARY KEY,
            topic      TEXT NOT NULL,
            cluster_id TEXT NOT NULL,
            title      TEXT,
            body       TEXT NOT NULL,
            priority   INTEGER NOT NULL,
            tags       TEXT,
            time       INTEGER NOT NULL,
            read       INTEGER NOT NULL DEFAULT 0,
            click      TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_msg_topic   ON messages(topic);
        CREATE INDEX IF NOT EXISTS idx_msg_cluster ON messages(cluster_id);
        "#,
    )?;
    Ok(conn)
}

// ---- settings ----

pub fn load_settings(conn: &Connection) -> Settings {
    let get = |k: &str| -> Option<String> {
        conn.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| r.get(0))
            .ok()
    };
    let mut s = Settings::default();
    if let Some(v) = get("server_url") {
        s.server_url = v;
    }
    if let Some(v) = get("token") {
        s.token = v;
    }
    if let Some(v) = get("topics") {
        s.topics = serde_json::from_str(&v).unwrap_or_default();
    }
    if let Some(v) = get("since") {
        s.since = v;
    }
    s
}

pub fn save_settings(conn: &Connection, s: &Settings) -> anyhow::Result<()> {
    let mut set = |k: &str, v: &str| -> anyhow::Result<()> {
        conn.execute(
            "INSERT INTO settings(key,value) VALUES(?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=?2",
            params![k, v],
        )?;
        Ok(())
    };
    set("server_url", &s.server_url)?;
    set("token", &s.token)?;
    set("topics", &serde_json::to_string(&s.topics)?)?;
    set("since", &s.since)?;
    Ok(())
}

pub fn save_since(conn: &Connection, since: &str) {
    let _ = conn.execute(
        "INSERT INTO settings(key,value) VALUES('since',?1)
         ON CONFLICT(key) DO UPDATE SET value=?1",
        [since],
    );
}

// ---- clusters ----

pub fn load_clusters(conn: &Connection) -> Vec<(String, String, Vec<f32>, u32)> {
    let mut out = Vec::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT topic, id, centroid, count FROM clusters")
    {
        let rows = stmt.query_map([], |r| {
            let topic: String = r.get(0)?;
            let id: String = r.get(1)?;
            let blob: Vec<u8> = r.get(2)?;
            let count: i64 = r.get(3)?;
            Ok((topic, id, blob_to_vec(&blob), count as u32))
        });
        if let Ok(rows) = rows {
            out.extend(rows.flatten());
        }
    }
    out
}

pub fn upsert_cluster(
    conn: &Connection,
    id: &str,
    topic: &str,
    label: &str,
    centroid: &[f32],
    count: u32,
    last_time: i64,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO clusters(id,topic,label,centroid,count,last_time)
         VALUES(?1,?2,?3,?4,?5,?6)
         ON CONFLICT(id) DO UPDATE SET centroid=?4, count=?5, last_time=?6",
        params![id, topic, label, vec_to_blob(centroid), count as i64, last_time],
    )?;
    Ok(())
}

// ---- messages ----

pub fn insert_message(conn: &Connection, m: &NtfyMessage, cluster_id: &str) -> anyhow::Result<()> {
    let tags = serde_json::to_string(&m.tags.clone().unwrap_or_default())?;
    conn.execute(
        "INSERT OR IGNORE INTO messages(id,topic,cluster_id,title,body,priority,tags,time,read,click)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,0,?9)",
        params![
            m.id,
            m.topic,
            cluster_id,
            m.title,
            m.message.clone().unwrap_or_default(),
            m.priority.unwrap_or(3),
            tags,
            m.time,
            m.click,
        ],
    )?;
    Ok(())
}

pub fn message_exists(conn: &Connection, id: &str) -> bool {
    conn.query_row("SELECT 1 FROM messages WHERE id=?1", [id], |_| Ok(()))
        .is_ok()
}

// ---- queries for UI ----

pub fn channels(conn: &Connection) -> Vec<ChannelDto> {
    let mut stmt = match conn.prepare(
        "SELECT topic,
                COUNT(*) AS total,
                SUM(CASE WHEN read=0 THEN 1 ELSE 0 END) AS unread,
                COUNT(DISTINCT cluster_id) AS clusters,
                MAX(time) AS last_time
         FROM messages GROUP BY topic ORDER BY last_time DESC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = stmt.query_map([], |r| {
        let topic: String = r.get(0)?;
        let last_time: Option<i64> = r.get(4)?;
        Ok((
            topic,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
            last_time,
        ))
    });
    let mut out = Vec::new();
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let (topic, total, unread, clusters, last_time) = row;
            let (last_title, last_body): (Option<String>, Option<String>) = conn
                .query_row(
                    "SELECT title, body FROM messages WHERE topic=?1 ORDER BY time DESC LIMIT 1",
                    [&topic],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap_or((None, None));
            out.push(ChannelDto {
                topic,
                total,
                unread,
                cluster_count: clusters,
                last_title,
                last_body,
                last_time,
            });
        }
    }
    out
}

pub fn clusters_for(conn: &Connection, topic: &str) -> Vec<ClusterDto> {
    let mut stmt = match conn.prepare(
        "SELECT cluster_id,
                COUNT(*) AS total,
                SUM(CASE WHEN read=0 THEN 1 ELSE 0 END) AS unread,
                MAX(time) AS last_time
         FROM messages WHERE topic=?1 GROUP BY cluster_id ORDER BY last_time DESC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = stmt.query_map([topic], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
        ))
    });
    let mut out = Vec::new();
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let (id, total, unread, last_time) = row;
            let label: String = conn
                .query_row("SELECT label FROM clusters WHERE id=?1", [&id], |r| r.get(0))
                .unwrap_or_default();
            let last_body: String = conn
                .query_row(
                    "SELECT body FROM messages WHERE cluster_id=?1 ORDER BY time DESC LIMIT 1",
                    [&id],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            out.push(ClusterDto {
                id,
                topic: topic.to_string(),
                label,
                total,
                unread,
                last_time,
                last_body,
            });
        }
    }
    out
}

pub fn messages_for(conn: &Connection, cluster_id: &str) -> Vec<MessageDto> {
    let mut stmt = match conn.prepare(
        "SELECT id,topic,cluster_id,title,body,priority,tags,time,read,click
         FROM messages WHERE cluster_id=?1 ORDER BY time DESC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = stmt.query_map([cluster_id], |r| {
        let tags_json: Option<String> = r.get(6)?;
        Ok(MessageDto {
            id: r.get(0)?,
            topic: r.get(1)?,
            cluster_id: r.get(2)?,
            title: r.get(3)?,
            body: r.get(4)?,
            priority: r.get(5)?,
            tags: tags_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            time: r.get(7)?,
            read: r.get::<_, i64>(8)? != 0,
            click: r.get(9)?,
        })
    });
    rows.map(|r| r.flatten().collect()).unwrap_or_default()
}

pub fn mark_read(conn: &Connection, id: &str) {
    let _ = conn.execute("UPDATE messages SET read=1 WHERE id=?1", [id]);
}

pub fn mark_cluster_read(conn: &Connection, cluster_id: &str) {
    let _ = conn.execute("UPDATE messages SET read=1 WHERE cluster_id=?1", [cluster_id]);
}

pub fn mark_channel_read(conn: &Connection, topic: &str) {
    let _ = conn.execute("UPDATE messages SET read=1 WHERE topic=?1", [topic]);
}

// ---- blob helpers (f32 <-> LE bytes) ----

fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    let mut b = Vec::with_capacity(v.len() * 4);
    for x in v {
        b.extend_from_slice(&x.to_le_bytes());
    }
    b
}

fn blob_to_vec(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
