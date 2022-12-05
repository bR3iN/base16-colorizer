use std::env;
use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use nom::character::complete::satisfy;
use nom::combinator::{map_res, recognize};
use nom::{IResult, Parser};

// The internally used error type and the corresponding `Result` alias.
pub(crate) type Error = Box<dyn std::error::Error>;
pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) trait Context<C, M, R>
where
    C: FnOnce() -> M,
{
    /// `Self` should represent a fallible type. In the case of failure, context should be
    /// evaluated and added to the failure's internal represenation.
    fn add_context(self, context: C) -> R;
}

// Implements `Context` for our `Result` alias and `std::result::Result<T, E>` where
// `E: std::error::Error`.
impl<T, E, C, M> Context<C, M, Result<T>> for std::result::Result<T, E>
where
    E: Into<Error>,
    C: FnOnce() -> M,
    M: std::fmt::Display,
{
    fn add_context(self, context: C) -> Result<T> {
        self.map_err(|err| format!("{}: {}", context(), err.into()).into())
    }
}

pub(crate) fn determine_home() -> std::result::Result<PathBuf, &'static str> {
    env::var("HOME").map(PathBuf::from).map_err(|_| {
        "Can't determine home directory. Make sure the environment variable 'HOME' is set."
    })
}

pub(crate) fn create_dir_recursive(path: impl AsRef<Path>) -> Result<()> {
    DirBuilder::new()
        .recursive(true)
        .create(path.as_ref())
        .add_context(|| format!("Failed to create {}", path.as_ref().display()))
}

pub(crate) fn read_into_buffer(path: impl AsRef<Path>) -> Result<String> {
    let mut buffer = String::new();

    File::open(&path)
        .add_context(|| format!("Failed to open {}", path.as_ref().display()))?
        .read_to_string(&mut buffer)
        .add_context(|| format!("While reading {}", path.as_ref().display()))?;

    Ok(buffer)
}

pub(crate) fn write_buffer(buffer: impl AsRef<str>, path: impl AsRef<Path>) -> Result<()> {
    File::create(&path)
        .add_context(|| format!("Failed to open {} for writing", path.as_ref().display()))?
        .write_all(buffer.as_ref().as_bytes())
        .add_context(|| format!("While writing {}", path.as_ref().display()))
}

pub(crate) fn hex_byte<'i, E>(input: &'i str) -> IResult<&'i str, u8, E>
where
    E: nom::error::ParseError<&'i str> + nom::error::FromExternalError<&'i str, ParseIntError>,
{
    map_res(recognize(hex_digit.and(hex_digit)), |s| {
        u8::from_str_radix(s, 16)
    })
    .parse(input)
}

fn hex_digit<'i, E>(input: &'i str) -> IResult<&str, char, E>
where
    E: nom::error::ParseError<&'i str>,
{
    satisfy(|c: char| c.is_digit(16)).parse(input)
}
