// src/memory/memory_data.rs

#[derive(Clone)]
pub struct Embedding(pub Vec<f32>);

#[derive(Clone)]
pub struct MemoryData(pub Embedding, pub String);
