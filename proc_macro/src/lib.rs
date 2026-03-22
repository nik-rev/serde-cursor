//! This crate is an implementation detail of the `serde_cursor` crate.

use proc_macro::Ident;
use proc_macro::Literal;
use proc_macro::Punct;
use proc_macro::Spacing;
use proc_macro::Span;
use proc_macro::TokenStream;
use proc_macro::TokenTree;

mod compile_error;
use compile_error::CompileError;
mod const_str;
mod path;

use path::Path;

#[proc_macro]
#[allow(nonstandard_style)]
pub fn CursorPath(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let cursor_path_segments = match parse_path_segments(&mut input, '+') {
        Ok(value) => value,
        Err(compile_error) => return compile_error,
    };

    let ident = match path::ident(&mut input) {
        Some(ident) => ident,
        None => {
            return CompileError::new(Span::call_site(), "expected identifier at the end").into()
        }
    };

    // Cursor path: `CursorPath<_, CursorPath<_, T>>`
    build_path(
        cursor_path_segments,
        TokenStream::from_iter([TokenTree::Ident(ident)]),
    )
}

#[proc_macro]
#[allow(nonstandard_style)]
pub fn Cursor(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    // These tokens make up the actual Type.
    //
    // Cursor!(a.0.c: HashMap<&str, &str>)
    //         ^^^^^
    let cursor_path_segments = match parse_path_segments(&mut input, ':') {
        Ok(value) => value,
        Err(compile_error) => return compile_error,
    };

    // These tokens make up the actual Type.
    //
    // Cursor!(a.0.c: HashMap<&str, &str>)
    //                 ^^^^^^^^^^^^^^^^^^^
    let type_tokens: TokenStream = if input.peek().is_none() {
        TokenStream::from_iter([ident("_")])
    } else {
        input.collect()
    };

    // Cursor path: `CursorPath<_, CursorPath<_, CursorPathEnd>>`
    let cursor_path = build_path(
        cursor_path_segments,
        TokenStream::from_iter([path([TokenTree::Ident(Ident::new(
            "CursorPathEnd",
            Span::call_site(),
        ))])]),
    );

    let mut ts = TokenStream::from_iter([
        punct(':'),
        punct(':'),
        ident("serde_cursor"),
        punct(':'),
        punct(':'),
        ident("Cursor"),
        punct('<'),
    ]);

    ts.extend(type_tokens);
    ts.extend([punct(',')]);
    ts.extend(cursor_path);
    ts.extend([punct('>')]);

    ts
}

fn build_path(cursor_path_segments: Vec<PathSegment>, end: TokenStream) -> TokenStream {
    cursor_path_segments
        .into_iter()
        .rev()
        .fold(end, |p, segment| {
            let segment = match segment {
                PathSegment::Interpolated { path, dollar: _ } => {
                    // Interpolated<P>

                    let mut ts = TokenStream::from_iter(path.into_tokens());

                    // Interpolated<CursorPath<...>>
                    //             ^
                    ts.extend([punct('<')]);

                    // Interpolated<CursorPath<...>>
                    //              ^^^^^^^^^^^^^^^
                    ts.extend(p);

                    // Interpolated<CursorPath<...>>
                    //                              ^
                    ts.extend([punct('>')]);

                    return ts;
                }
                PathSegment::Field { value, spans } => const_str::encode(&value, &spans),
                PathSegment::Index { value: index, span } => {
                    let mut ts = path([ident("Index")]);
                    ts.extend([punct('<')]);
                    let mut lit = Literal::u128_unsuffixed(index);
                    lit.set_span(span);
                    ts.extend(Some(TokenTree::Literal(lit)));
                    ts.extend([punct('>')]);
                    ts
                }
                PathSegment::Wildcard(_span) => path([ident("Wildcard")]),
            };

            let mut ts = path([ident("CursorPath")]);
            ts.extend([punct('<')]);
            ts.extend(segment);
            ts.extend([punct(',')]);
            ts.extend(p);
            ts.extend([punct('>')]);
            ts
        })
}

fn parse_path_segments(
    input: &mut std::iter::Peekable<proc_macro::token_stream::IntoIter>,
    end_token: char,
) -> Result<Vec<PathSegment>, TokenStream> {
    let mut cursor_path_segments = Vec::new();
    let mut started = false;
    while let Some(tt) = input.peek() {
        // the "." is not required for the first path
        //
        // Cursor!(a.0.c: bool)
        //         ^
        if !started {
            match parse_path_segment(input) {
                Ok(seg) => cursor_path_segments.push(seg),
                Err(e) => return Err(e.into()),
            }

            started = true;

            continue;
        }

        match tt {
            // Path ends at a colon
            //
            // Cursor!(a.b.c: bool)
            //               ^
            TokenTree::Punct(p) if p.as_char() == end_token => {
                input.next();

                break;
            }
            // A single path segment
            //
            // Cursor!(a.0.c: bool)
            //          ^^
            TokenTree::Punct(p) if p.as_char() == '.' => {
                input.next();

                match parse_path_segment(input) {
                    Ok(seg) => cursor_path_segments.push(seg),
                    Err(e) => return Err(e.into()),
                }
            }
            _ => break,
        }
    }
    Ok(cursor_path_segments)
}

/// Represents a single segment of a path.
#[derive(Debug)]
enum PathSegment {
    /// The Wildcard, `*`.
    ///
    /// ```txt
    /// Cursor!(packages.*.name)
    ///                  ^
    /// ```
    ///
    /// The `Span` is of the `*` token.
    Wildcard(Span),
    /// An interpolated path segment
    ///
    /// ```txt
    /// type Deps<T> = CursorPath!(*.dependencies.$T);
    ///
    /// Cursor!(package.$Deps.0)
    ///                 ^^^^^
    /// ```
    Interpolated {
        /// Path to the generic type itself.
        ///
        /// ```txt
        /// Cursor!(package.$Deps.0)
        ///                 ^^^^^
        /// ```
        path: Path,
        /// Span of the dollar.
        ///
        /// ```txt
        /// Cursor!(package.$Deps.0)
        ///                 ^
        /// ```
        #[expect(unused)]
        dollar: Span,
    },
    /// Field with a name.
    Field {
        /// Actual string value of the field.
        ///
        /// ```txt
        /// Cursor!(foo.bar-baz---quux)
        /// ```
        ///
        /// The `value` of the 2nd field there is "bar-baz---quux"
        value: String,
        /// Spans of all `-` tokens and identifiers that are part of this field.
        ///
        /// This `Vec` is non-empty.
        ///
        /// Every consecutive series of ^ represents a single span:
        ///
        /// ```txt
        /// Cursor!(foo.bar-baz---quux)
        ///             ^^^
        ///                ^
        ///                 ^^^
        ///                    ^
        ///                     ^
        ///                      ^
        ///                       ^^^^
        /// ```
        ///
        /// We collect these spans because we want the field
        /// to be syntax-highlighted as a single entity
        /// (works for IDEs that support semantic highlighting)
        spans: Vec<Span>,
    },
    /// Index into a sequence.
    ///
    /// ```txt
    /// Cursor!(packages.*.dependencies.0)
    ///                                 ^
    /// ```
    Index {
        /// Integer value of the index, in the above case it is `0`.
        value: u128,
        /// Span of the integer literal.
        span: Span,
    },
}

fn parse_path_segment(
    input: &mut std::iter::Peekable<proc_macro::token_stream::IntoIter>,
) -> Result<PathSegment, CompileError> {
    let tt = input.peek().ok_or_else(|| {
        CompileError::new(
            Span::call_site(),
            "expected path segment, found end of input",
        )
    })?;

    match tt {
        // Identifier fields
        //
        // Cursor!(a.b-c---d.c: bool)
        //         ^
        tt if matches!(tt, TokenTree::Ident(_))
            || matches!(tt, TokenTree::Punct(p) if p.as_char() == '-') =>
        {
            let tt = input.next().unwrap();

            let mut field = tt.to_string();

            let mut spans = Vec::from([tt.span()]);

            loop {
                match input.peek() {
                    Some(TokenTree::Ident(ident)) => {
                        spans.push(ident.span());
                        field.push_str(&ident.to_string());
                        input.next().unwrap();
                    }
                    Some(TokenTree::Punct(p)) if p.as_char() == '-' => {
                        spans.push(p.span());
                        field.push('-');
                        input.next().unwrap();
                    }
                    _ => break,
                }
            }

            Ok(PathSegment::Field {
                value: field,
                spans,
            })
        }
        TokenTree::Punct(p) if p.as_char() == '$' => {
            let dollar = p.span();
            let _ = input.next();

            let path = Path::parse(input)?;

            Ok(PathSegment::Interpolated { path, dollar })
        }
        TokenTree::Punct(p) if p.as_char() == '*' => {
            let span = p.span();
            let _ = input.next();
            Ok(PathSegment::Wildcard(span))
        }
        TokenTree::Literal(lit) => {
            let span = lit.span();
            match litrs::Literal::from(lit) {
                // Integer index
                //
                // Cursor!(a.0.c: bool)
                //           ^
                litrs::Literal::Integer(index) => {
                    let val = index
                        .value::<u128>()
                        .ok_or_else(|| CompileError::new(span, "invalid integer index"))?;
                    input.next();
                    Ok(PathSegment::Index { value: val, span })
                }
                // Integer index
                //
                // Cursor!(a."hello world".c: bool)
                //           ^^^^^^^^^^^^^
                litrs::Literal::String(field) => {
                    let val = field.value().to_string();
                    input.next();
                    Ok(PathSegment::Field {
                        value: val,
                        spans: Vec::from([span]),
                    })
                }
                _ => {
                    Err(CompileError::new(
                        span,
                        "expected identifier, '*', integer, or string",
                    ))
                }
            }
        }
        _ => Err(CompileError::new(tt.span(), "unexpected token")),
    }
}

fn ident(name: &str) -> TokenTree {
    TokenTree::Ident(Ident::new(name, Span::call_site()))
}

fn punct(char: char) -> TokenTree {
    TokenTree::Punct(Punct::new(char, Spacing::Joint))
}

/// Returns path at `::serde_cursor`
fn path(segments: impl IntoIterator<Item = TokenTree>) -> TokenStream {
    segments.into_iter().enumerate().fold(
        TokenStream::from_iter([
            punct(':'),
            punct(':'),
            ident("serde_cursor"),
            punct(':'),
            punct(':'),
        ]),
        |mut ts, (i, path_segment)| {
            if i > 0 {
                ts.extend([punct(':'), punct(':')]);
            }
            ts.extend(Some(path_segment));
            ts
        },
    )
}
