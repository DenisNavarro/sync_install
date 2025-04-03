#[macro_export]
macro_rules! newtype {
    ($ty:ident, error_msg = $error_msg:expr $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ty<'a>(&'a str);

        impl<'a> $ty<'a> {
            pub fn from_str(value: &'a str) -> anyhow::Result<Self> {
                anyhow::ensure!(!value.is_empty(), $error_msg);
                Ok(Self(value))
            }
            #[inline]
            pub const fn as_str(self) -> &'a str {
                self.0
            }
        }
    };
}
pub use newtype;
