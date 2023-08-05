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

pub trait SeqGen {
    fn size() -> usize;
    fn seq() -> Self;
}

macro_rules! tuple_of_usize {
    ($($idx:tt$($dummy:ident)*,)*) => {
        ($(usize$($dummy:ident)*,)*)
    };
}
macro_rules! rev_tuple {
    ((), ($($result:tt)*)) => {
        ($($result)*)
    };
    (($head:tt, $($tail:tt,)*) , ($($result:tt)*)) => {
        rev_tuple!(($($tail,)*), ($head, $($result)*))
    };
}

macro_rules! gen_seq_gen {
    () => {};
    ($size:tt, $($idx:tt,)*) => {
        gen_seq_gen!($($idx,)*);
        impl SeqGen for tuple_of_usize!($($idx,)*) {
            fn size() -> usize {
                $size
            }
            fn seq() -> Self {
                rev_tuple!(($($idx,)*), ())
            }
        }
    };
}

gen_seq_gen!(10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,);

#[test]
fn test_seq_gen() {
    assert_eq!(<(usize, usize, usize) as SeqGen>::size(), 3);
    assert_eq!(<(usize, usize, usize) as SeqGen>::seq(), (0, 1, 2));
}
