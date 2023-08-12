// Compiler Context

use std::sync::atomic::{self, AtomicUsize};
use std::sync::Arc;

#[derive(Debug)]
pub struct CCtx {
    pub id_gen: IdGen,
}

impl CCtx {
    pub fn new() -> Self {
        Self {
            id_gen: IdGen::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdGen {
    next_id: Arc<AtomicUsize>,
}

impl IdGen {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicUsize::new(1)),
        }
    }

    pub fn fresh(&self) -> Id {
        Id {
            number: self.next_id.fetch_add(1, atomic::Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Id {
    pub number: usize,
}

impl Id {
    pub fn dummy() -> Self {
        Id::default()
    }

    pub fn is_dummy(&self) -> bool {
        self.number == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_gen() {
        let id_gen = IdGen::new();
        assert_eq!(id_gen.fresh(), Id { number: 1 });
        assert_eq!(id_gen.fresh(), Id { number: 2 });
        assert_eq!(id_gen.fresh(), Id { number: 3 });
    }
}
