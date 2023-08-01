pub trait RtCtx {
    fn puts(&self, s: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RtCtxImpl;

impl RtCtx for RtCtxImpl {
    fn puts(&self, s: &str) {
        println!("{}", s);
    }
}
