pub(crate) trait SeqInit {
    fn seq() -> Self;
}

impl<const N: usize> SeqInit for [usize; N] {
    fn seq() -> Self {
        let mut arr = [0; N];
        for i in 0..N {
            arr[i] = i;
        }
        arr
    }
}
