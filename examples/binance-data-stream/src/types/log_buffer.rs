use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use chrono::Local;

#[derive(Clone)]
pub struct LogBuffer {
    inner: Arc<Mutex<VecDeque<String>>>,
    capacity: usize,
}

impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        let capacity = if capacity < 1000 { 1000 } else { capacity };
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    pub fn push(&self, msg: String) {
        let now = Local::now();
        let timestamp = now.format("%H:%M:%S%.3f");
        // Quebra linha automática para mensagens longas
        let formatted = format!("[{}] {}", timestamp, msg.replace("\n", "\n   "));
        let mut inner = self.inner.lock().unwrap();
        if inner.len() == self.capacity {
            inner.pop_front();
        }
        inner.push_back(formatted);
    }

    pub fn get_all(&self) -> Vec<String> {
        let inner = self.inner.lock().unwrap();
        // Retorna as últimas mensagens primeiro
        inner.iter().rev().cloned().collect()
    }
}
