// Remark (2024-07-07): the nonempty collections and iterators from the `mitsein` crate look good,
// but I prefer to avoid to add another dependency right now.

use std::fmt;

use anyhow::{Context as _, ensure};

#[derive(Clone, PartialEq, Eq)]
pub struct Command<'a>(Vec<&'a str>);

impl<'a> Command<'a> {
    fn ensure_invariant(program_and_args: &[&'a str]) -> anyhow::Result<()> {
        let program = program_and_args.first().context("missing program")?;
        ensure!(!program.is_empty(), "empty program");
        Ok(())
    }
    pub fn from_vec(program_and_args: Vec<&'a str>) -> anyhow::Result<Self> {
        Self::ensure_invariant(&program_and_args)?;
        Ok(Self(program_and_args))
    }
    pub fn from_str(program_and_args: &'a str) -> anyhow::Result<Self> {
        // I don't need `shlex::split` for my use case.
        Self::from_vec(program_and_args.split(' ').collect())
    }
    #[cfg(test)]
    #[inline]
    pub fn into_vec(self) -> Vec<&'a str> {
        self.0
    }
    pub fn split_program_and_args(&self) -> (&'a str, &[&'a str]) {
        // There is at least one element so `unwrap()` is OK.
        let (program, args) = self.0.split_first().unwrap();
        (*program, args)
    }
    pub fn concat_args(&self, args: impl IntoIterator<Item = &'a str>) -> Self {
        Self(self.0.iter().copied().chain(args).collect())
    }
    pub fn display(&self) -> impl fmt::Display {
        shlex::try_join(self.0.iter().copied()).unwrap()
    }
}

#[macro_export]
macro_rules! command {
    ($($x:expr),+ $(,)?) => {
        $crate::command::Command::from_vec(std::vec![$($x),+])
    };
}
pub use command;
