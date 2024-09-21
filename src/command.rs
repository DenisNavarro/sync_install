// Remark (2024-07-07): the nonempty collections and iterators from the `mitsein` crate look good,
// but I prefer to avoid to add another dependency right now.

use std::borrow::Cow;
use std::fmt;

use anyhow::{ensure, Context};
use itertools::Itertools; // format

#[derive(Clone, PartialEq, Eq)]
pub struct Command<'a>(Vec<Cow<'a, str>>);

impl<'a> Command<'a> {
    fn ensure_invariant(program_and_args: &[Cow<'a, str>]) -> anyhow::Result<()> {
        let program = program_and_args.first().context("missing program")?;
        ensure!(!program.is_empty(), "empty program");
        Ok(())
    }
    pub fn from_vec(program_and_args: Vec<Cow<'a, str>>) -> anyhow::Result<Self> {
        Self::ensure_invariant(&program_and_args)?;
        Ok(Self(program_and_args))
    }
    pub fn from_str(program_and_args: &'a str) -> anyhow::Result<Self> {
        // I don't need `shlex::split` for my use case.
        Self::from_vec(program_and_args.split(' ').map(Into::into).collect())
    }
    #[allow(clippy::allow_attributes)]
    #[allow(dead_code)] // False positive: called in `happy_path_tests.rs`
    #[inline]
    pub fn into_vec(self) -> Vec<Cow<'a, str>> {
        self.0
    }
    pub fn split_program_and_args(&self) -> (&Cow<'a, str>, &[Cow<'a, str>]) {
        // There is at least one element so `unwrap()` is OK.
        self.0.split_first().unwrap()
    }
    pub fn concat_args(&self, args: impl IntoIterator<Item = Cow<'a, str>>) -> Self {
        Self(self.0.iter().cloned().chain(args).collect())
    }
    pub fn format(&self) -> impl fmt::Display + '_ {
        // I don't need `shlex::try_join` for my use case.
        self.0.iter().format(" ")
    }
}

#[macro_export]
macro_rules! command {
    ($($x:expr),+ $(,)?) => {
        $crate::command::Command::from_vec(std::vec![$(std::borrow::Cow::from($x)),+])
    };
}
pub use command;
