use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char, u8};
use nom::combinator::{flat_map, map_res, success};
use nom::error::VerboseError;
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded, terminated};
use nom::{Finish, Parser};

use crate::color::Color;
use crate::scheme::ColorScheme;
use crate::utils::{hex_byte, Result};

pub(crate) fn apply_colors(scheme: &ColorScheme, input: &str) -> Result<String> {
    // FIXME: This reads the whole file into memory; buffered reading in combination with
    // `nom`'s streaming parsers might be more desirable here. In this case, we also want to
    // delete the half-written file in case an error is encountered in the file.

    Ok(fold_many0(token(scheme), String::new, |mut buf, token| {
        match token {
            Token::Color(c) => buf.push_str(&c.to_string()),
            Token::Char(c) => buf.push(c),
            Token::Literal(s) => buf.push_str(s),
        };
        buf
    })
    .parse(input.as_ref())
    .finish()
    .map(|(_, res)| res)
    .map_err(|err| format!("{}", err))?)
}

#[derive(Debug)]
enum Token<'t> {
    Color(Color),
    Char(char),
    Literal(&'t str),
}

type IResult<'i, O> = nom::IResult<&'i str, O, VerboseError<&'i str>>;

fn token<'s>(scheme: &'s ColorScheme) -> impl FnMut(&str) -> IResult<Token<'s>> + '_ {
    move |input| alt((non_char_token(scheme), anychar.map(Token::Char))).parse(input)
}

fn percentage(input: &str) -> IResult<u8> {
    terminated(u8, char('%')).parse(input)
}

fn color(scheme: &ColorScheme) -> impl FnMut(&str) -> IResult<Color> + '_ {
    move |input| {
        let base_color = map_res(delimited(tag("base"), hex_byte, tag("-hex")), |nr: u8| {
            scheme
                .colors
                .get(nr as usize)
                .ok_or_else(|| "Color number must be between 00 and 0F")
                .map(Clone::clone)
        });

        flat_map(base_color, |c: Color| {
            move |input| {
                alt((
                    preceded(tag(";+"), percentage).map(|p| c.lighten(p)),
                    preceded(tag(";-"), percentage).map(|p| c.darken(p)),
                    success(c),
                ))
                .parse(input)
            }
        })
        .parse(input)
    }
}

fn non_char_token<'s>(scheme: &'s ColorScheme) -> impl FnMut(&str) -> IResult<Token<'s>> {
    move |input| {
        delimited(
            tag("{{"),
            alt((
                tag("scheme-author")
                    .map(|_| &scheme.author as &str)
                    .map(Token::Literal),
                tag("scheme-name")
                    .map(|_| &scheme.name as &str)
                    .map(Token::Literal),
                color(scheme).map(Token::Color),
            )),
            tag("}}"),
        )
        .parse(input)
    }
}
