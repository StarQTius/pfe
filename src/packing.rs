pub trait Pack {
    type Input;
    type Output;

    fn pack_inplace<const IN_N: usize, const OUT_N: usize>(
        &self,
        f: &impl Fn(&[Self::Input; IN_N]) -> [Self::Output; OUT_N],
        output: &mut [Self::Output],
    );

    fn pack<const IN_N: usize, const OUT_N: usize, const PACKED_N: usize>(
        &self,
        f: &impl Fn(&[Self::Input; IN_N]) -> [Self::Output; OUT_N],
    ) -> [Self::Output; PACKED_N]
    where
        Self::Output: Default + Copy,
    {
        let mut retval = [Self::Output::default(); PACKED_N];

        self.pack_inplace(f, &mut retval);

        retval
    }

    fn unpack<const IN_N: usize, const OUT_N: usize>(
        packed: &[Self::Output],
        f: &impl Fn(&[Self::Output; OUT_N]) -> [Self::Input; IN_N],
    ) -> Self;
}
