#[macro_export]
macro_rules! newtype {
    ($type_name:ident, error_msg = $error_msg:expr) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $type_name<'a>(&'a str);

        impl<'a> $type_name<'a> {
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
