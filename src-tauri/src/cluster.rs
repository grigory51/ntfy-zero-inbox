use std::collections::HashMap;

/// Абстракция эмбеддера — чтобы десктоп мог использовать локальную модель,
/// а слабое/оффлайн-окружение (и iOS в перспективе) — резервный вариант.
pub trait Embedder: Send {
    fn dim(&self) -> usize;
    fn embed(&self, text: &str) -> Vec<f32>;
}

/// Резервный эмбеддер без модели: хеширование токенов в фиксированный вектор.
/// Кластеризует по пересечению словаря — грубо, но работает всегда и мгновенно.
pub struct HashingEmbedder {
    dim: usize,
}

impl HashingEmbedder {
    pub fn new() -> Self {
        HashingEmbedder { dim: 256 }
    }
}

impl Embedder for HashingEmbedder {
    fn dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        let mut v = vec![0.0f32; self.dim];
        for tok in text
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() > 1)
        {
            let mut h: u64 = 1469598103934665603;
            for b in tok.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(1099511628211);
            }
            let idx = (h as usize) % self.dim;
            v[idx] += 1.0;
        }
        normalize(&mut v);
        v
    }
}

/// Локальная модель эмбеддингов (fastembed / ONNX, CPU). Недоступна на мобилках.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
pub struct FastEmbedder {
    model: fastembed::TextEmbedding,
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
impl FastEmbedder {
    /// Пытается поднять all-MiniLM-L6-v2 (~23 МБ, скачивается один раз).
    pub fn try_new() -> anyhow::Result<Self> {
        use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(false),
        )?;
        Ok(FastEmbedder { model })
    }
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
impl Embedder for FastEmbedder {
    fn dim(&self) -> usize {
        384
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        match self.model.embed(vec![text], None) {
            Ok(mut batch) if !batch.is_empty() => {
                let mut v = batch.swap_remove(0);
                normalize(&mut v);
                v
            }
            _ => vec![0.0; 384],
        }
    }
}

struct Centroid {
    id: String,
    vec: Vec<f32>,
    count: u32,
}

pub struct Assignment {
    pub cluster_id: String,
    pub centroid: Vec<f32>,
    pub count: u32,
}

/// Онлайн-кластеризация: жадное присоединение к ближайшему центроиду
/// внутри канала по косинусной близости; иначе — новый кластер.
pub struct Clusterer {
    embedder: Box<dyn Embedder>,
    clusters: HashMap<String, Vec<Centroid>>,
    threshold: f32,
    pub model_ready: bool,
}

impl Clusterer {
    pub fn new(embedder: Box<dyn Embedder>) -> Self {
        Clusterer {
            embedder,
            clusters: HashMap::new(),
            threshold: 0.6,
            model_ready: false,
        }
    }

    /// Подгрузка сохранённых центроидов при старте (из SQLite).
    pub fn load_centroid(&mut self, topic: &str, id: String, vec: Vec<f32>, count: u32) {
        if vec.len() != self.embedder.dim() {
            return; // размерность сменилась (другой эмбеддер) — пропускаем
        }
        self.clusters
            .entry(topic.to_string())
            .or_default()
            .push(Centroid { id, vec, count });
    }

    /// Замена эмбеддера (когда доподнялась локальная модель). Центроиды
    /// сбрасываются: новые сообщения пойдут кластеризоваться моделью.
    pub fn set_embedder(&mut self, embedder: Box<dyn Embedder>) {
        self.embedder = embedder;
        self.clusters.clear();
        self.model_ready = true;
    }

    /// Убрать центроид удалённого кластера, чтобы новые сообщения к нему не липли.
    pub fn remove_cluster(&mut self, id: &str) {
        for v in self.clusters.values_mut() {
            v.retain(|c| c.id != id);
        }
    }

    /// Убрать все центроиды канала.
    pub fn remove_topic(&mut self, topic: &str) {
        self.clusters.remove(topic);
    }

    pub fn assign(&mut self, topic: &str, text: &str) -> Assignment {
        let v = self.embedder.embed(text);
        let entry = self.clusters.entry(topic.to_string()).or_default();

        let mut best_i = None;
        let mut best_sim = -1.0f32;
        for (i, c) in entry.iter().enumerate() {
            let s = dot(&c.vec, &v);
            if s > best_sim {
                best_sim = s;
                best_i = Some(i);
            }
        }

        match best_i {
            Some(i) if best_sim >= self.threshold => {
                let c = &mut entry[i];
                let n = c.count as f32;
                for k in 0..c.vec.len() {
                    c.vec[k] = (c.vec[k] * n + v[k]) / (n + 1.0);
                }
                normalize(&mut c.vec);
                c.count += 1;
                Assignment {
                    cluster_id: c.id.clone(),
                    centroid: c.vec.clone(),
                    count: c.count,
                }
            }
            _ => {
                let id = uuid::Uuid::new_v4().to_string();
                entry.push(Centroid {
                    id: id.clone(),
                    vec: v.clone(),
                    count: 1,
                });
                Assignment {
                    cluster_id: id,
                    centroid: v,
                    count: 1,
                }
            }
        }
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-8 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}
