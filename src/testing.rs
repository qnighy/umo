use crate::rt_ctx::RtCtx;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MockRtCtx {
    pub stdout: Arc<Mutex<String>>,
}

impl MockRtCtx {
    pub fn new() -> Self {
        Self {
            stdout: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl RtCtx for MockRtCtx {
    fn puts(&self, s: &str) {
        let mut stdout = self.stdout.lock().unwrap();
        stdout.push_str(s);
        stdout.push('\n');
    }
}
