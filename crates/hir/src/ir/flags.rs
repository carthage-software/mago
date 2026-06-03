use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(into = "u32")]
pub struct Flags<T: Clone + Into<u32>>(u32, std::marker::PhantomData<T>);

impl<T: Clone + Into<u32>> Flags<T> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(0, std::marker::PhantomData)
    }

    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits, std::marker::PhantomData)
    }

    #[inline]
    pub fn set(&mut self, flag: T) {
        self.0 |= flag.into();
    }

    #[inline]
    pub fn unset(&mut self, flag: T) {
        self.0 &= !flag.into();
    }

    #[inline]
    #[must_use]
    pub fn is_set(&self, flag: T) -> bool {
        (self.0 & flag.into()) != 0
    }

    #[inline]
    pub const fn clear(&mut self) {
        self.0 = 0;
    }

    #[inline]
    #[must_use]
    pub const fn bits(&self) -> u32 {
        self.0
    }
}

impl<T: Clone + Into<u32>> From<Flags<T>> for u32 {
    #[inline]
    fn from(flags: Flags<T>) -> Self {
        flags.0
    }
}

impl<T: Clone + Into<u32>> From<u32> for Flags<T> {
    #[inline]
    fn from(bits: u32) -> Self {
        Self::from_bits(bits)
    }
}

impl<T: Clone + Into<u32>> std::ops::BitOr for Flags<T> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.0 | rhs.0)
    }
}

impl<T: Clone + Into<u32>> std::ops::BitOrAssign for Flags<T> {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl<T: Clone + Into<u32>> std::ops::BitAnd for Flags<T> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.0 & rhs.0)
    }
}

impl<T: Clone + Into<u32>> std::ops::BitAndAssign for Flags<T> {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl<T: Clone + Into<u32>> std::ops::BitXor for Flags<T> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.0 ^ rhs.0)
    }
}

impl<T: Clone + Into<u32>> std::ops::BitXorAssign for Flags<T> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl<T: Clone + Into<u32>> std::default::Default for Flags<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
