//! This module implements parsing for Rust paths via the [`Path`] type.

use std::iter::Peekable;

use proc_macro::token_stream;
use proc_macro::Ident;
use proc_macro::Punct;
use proc_macro::Spacing;
use proc_macro::Span;
use proc_macro::TokenTree;

use crate::CompileError;

/// A path to an item
///
/// Example: `::std::hash::Hash`
#[derive(Clone, Debug)]
pub struct Path {
    /// `true` if the leading colon is present
    ///
    /// ```txt
    /// ::std::hash::Hash
    /// ^^
    /// ```
    pub leading_colon: Option<PathSeparator>,
    /// First component of the path
    ///
    /// ```txt
    /// ::std::hash::Hash
    ///   ^^^
    /// ```
    pub first_component: Ident,
    /// Other components of the path
    ///
    /// ```txt
    /// ::std::hash::Hash
    ///      ^^^^^^
    ///            ^^^^^^
    /// ```
    pub components: Vec<(PathSeparator, Ident)>,
}

/// Separator in a path: `::`
///
/// ```ignore
/// ::std::hash::Hash
/// ^^   ^^    ^^
/// ```
#[derive(Clone, Debug)]
pub struct PathSeparator {
    /// Span of the first `:`
    ///
    /// ```ignore
    /// ::std::hash::Hash
    /// ^    ^     ^
    /// ```
    pub first: Span,
    /// Span of the second `:`
    ///
    /// ```ignore
    /// ::std::hash::Hash
    ///  ^    ^     ^
    /// ```
    pub second: Span,
}

impl PathSeparator {
    fn parse(ts: &mut Peekable<token_stream::IntoIter>) -> Option<Self> {
        match ts.peek() {
            Some(TokenTree::Punct(colon)) if *colon == ':' => {
                let span = colon.span();
                ts.next();

                match ts.next() {
                    Some(TokenTree::Punct(colon_colon)) if colon_colon == ':' => {
                        Some(PathSeparator {
                            first: span,
                            second: colon_colon.span(),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn into_tokens(self) -> impl Iterator<Item = TokenTree> {
        let mut first = Punct::new(':', Spacing::Joint);
        first.set_span(self.first);
        let mut second = Punct::new(':', Spacing::Joint);
        second.set_span(self.second);
        [TokenTree::Punct(first), TokenTree::Punct(second)].into_iter()
    }
}

impl Path {
    /// Expect a path.
    ///
    /// Requires the `ts` to start with `::segment` (absolute) or `segment` (relative), then
    /// expects 0 or more `::segment`s followed by whatever (once we hit "whatever", stops trying to parse further)
    ///
    /// `start_span` is the span of the thing directly before the `Path`
    pub fn parse(ts: &mut Peekable<token_stream::IntoIter>) -> Result<Self, CompileError> {
        // Parse beginning of the path

        // ::std::hash::Hash
        // ^^
        let leading_colon = PathSeparator::parse(ts);

        // ::std::hash::Hash
        //   ^^^
        let first_component =
            ident(ts).ok_or_else(|| CompileError::new(Span::call_site(), "invalid path"))?;

        let mut components = Vec::new();

        // Parses rest of the path. Each segment is preceded by the path separator `::`
        //
        // ::std::hash::Hash
        //      ^^            iteration 1
        //        ^^^^
        //
        //            ^^      iteration 2
        //              ^^^^
        loop {
            match ts.peek() {
                Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                    // finished parsing path
                    break;
                }
                _ => {}
            }

            let separator = PathSeparator::parse(ts)
                .ok_or_else(|| CompileError::new(Span::call_site(), "invalid path"))?;

            let component =
                ident(ts).ok_or_else(|| CompileError::new(Span::call_site(), "invalid path"))?;

            components.push((separator, (component)));
        }

        Ok(Path {
            leading_colon,
            first_component,
            components,
        })
    }

    pub fn into_tokens(self) -> impl Iterator<Item = TokenTree> {
        self.leading_colon
            .map(PathSeparator::into_tokens)
            .into_iter()
            .flatten()
            // first path segment
            //
            // ::core::hash::Hash
            //   ^^^
            .chain([TokenTree::Ident(self.first_component)])
            .chain(
                self.components
                    .into_iter()
                    .flat_map(|(separator, segment)| {
                        // path separator `::`
                        //
                        // ::std::hash::HashMap
                        //      ^^    ^^
                        separator
                            .into_tokens()
                            // path segment
                            //
                            // ::std::hash::HashMap
                            //        ^^^^  ^^^^^^^
                            .chain(std::iter::once(TokenTree::Ident(segment)))
                    }),
            )
    }
}

/// Get the next token punctuation if it is an identifier
pub fn ident(ts: &mut Peekable<token_stream::IntoIter>) -> Option<Ident> {
    match ts.peek() {
        Some(TokenTree::Ident(_ident)) => {
            let Some(TokenTree::Ident(ident)) = ts.next() else {
                unreachable!(".peek() returned `Some(TokenTree::Ident)`")
            };

            Some(ident)
        }
        _ => None,
    }
}
