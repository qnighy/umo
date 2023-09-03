use std::fmt;

pub trait PDebug<P: Copy> {
    fn pfmt(&self, f: &mut fmt::Formatter<'_>, p: P) -> fmt::Result;
}

pub trait PDebugExt<P: Copy>: PDebug<P> {
    fn debug_with<'a>(&'a self, p: P) -> PDebugWrapper<'a, Self, P> {
        PDebugWrapper { value: self, p }
    }
}

impl<T, P> PDebugExt<P> for T
where
    T: PDebug<P> + ?Sized,
    P: Copy,
{
}

pub struct PDebugWrapper<'a, T, P>
where
    T: PDebug<P> + ?Sized,
    P: Copy,
{
    value: &'a T,
    p: P,
}

impl<'a, T, P> fmt::Debug for PDebugWrapper<'a, T, P>
where
    T: PDebug<P> + ?Sized,
    P: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.pfmt(f, self.p)
    }
}

pub fn debug_with<F>(f: F) -> DebugWith<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    DebugWith { f }
}

pub struct DebugWith<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    f: F,
}

impl<F> fmt::Debug for DebugWith<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.f)(f)
    }
}

pub fn debug_with_display<T>(value: T) -> DebugWithDisplay<T>
where
    T: fmt::Display,
{
    DebugWithDisplay { value }
}

pub struct DebugWithDisplay<T: fmt::Display> {
    value: T,
}

impl<T: fmt::Display> fmt::Debug for DebugWithDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.value, f)
    }
}

impl<T, P> PDebug<P> for Vec<T>
where
    T: PDebugExt<P>,
    P: Copy,
{
    fn pfmt(&self, f: &mut fmt::Formatter<'_>, params: P) -> fmt::Result {
        f.debug_list()
            .entries(self.iter().map(|x| x.debug_with(params)))
            .finish()
    }
}
