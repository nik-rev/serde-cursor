//! This crate is an implementation detail of the [`serde_cursor`](https://docs.rs/serde_cursor/latest/serde_cursor) crate.

use proc_macro::Delimiter;
use proc_macro::Group;
use proc_macro::Ident;
use proc_macro::Punct;
use proc_macro::Spacing;
use proc_macro::Span;
use proc_macro::TokenStream;
use proc_macro::TokenTree;

mod compile_error;
use compile_error::CompileError;
mod const_str;
mod path;

/// Access nested fields of serde-compatible data formats easily.
///
/// This macro expands to a type implementing [`serde::Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html).
///
/// # Example
///
/// Get version from `Cargo.toml`:
///
/// ```
/// use serde_cursor::Cursor;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = r#"
///     [workspace.package]
///     version = "0.1"
/// "#;
///
/// let version: String = toml::from_str::<Cursor!(workspace.package.version)>(data)?.0;
/// assert_eq!(version, "0.1");
/// # Ok(()) }
/// ```
///
/// See the [crate-level](https://docs.rs/serde_cursor/latest/serde_cursor) documentation for more info.
#[proc_macro]
#[allow(nonstandard_style)]
pub fn Cursor(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    // These tokens make up the actual Type.
    //
    // Cursor!(a[0].c: HashMap<&str, &str>)
    //         ^^^^^^
    let cursor_path_segments = match parse_path_segments(&mut input, ':') {
        Ok(value) => value,
        Err(compile_error) => return compile_error,
    };

    // These tokens make up the actual Type.
    //
    // Cursor!(a[0].c: HashMap<&str, &str>)
    //                 ^^^^^^^^^^^^^^^^^^^
    let type_tokens: TokenStream = if input.peek().is_none() {
        TokenStream::from_iter([ident("_")])
    } else {
        input.collect()
    };

    // Cursor path: `Path<_, Path<_, PathEnd>>`
    let cursor_path = build_path(
        cursor_path_segments,
        TokenStream::from_iter([path([TokenTree::Ident(Ident::new(
            "PathEnd",
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

    // panic!("{ts}");

    ts
}

/// Support for interpolations, `Cursor!(japan.$Details.air_temperature)`.
///
/// # Example
///
/// It's not uncommon for multiple queries to get quite repetitive:
///
/// ```
/// # use serde_json::from_str;
/// # use serde_cursor::Cursor;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let france = "france = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 1.0, relative_humidity = 2.0, air_temperature = 3.0 } } } }] } }";
/// # let japan = "japan = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 4.0, relative_humidity = 5.0, air_temperature = 6.0 } } } }] } }";
/// let pressure: Vec<f64> = toml::from_str::<Cursor!(france.properties.timeseries[].data.instant.details.air_pressure_at_sea_level)>(france)?.0;
/// let humidity: Vec<f64> = toml::from_str::<Cursor!(japan.properties.timeseries[].data.instant.details.relative_humidity)>(japan)?.0;
/// let temperature: Vec<f64> = toml::from_str::<Cursor!(japan.properties.timeseries[].data.instant.details.air_temperature)>(japan)?.0;
/// # Ok(()) }
/// ```
///
/// `serde_cursor` supports **interpolations**. You can factor out the common path into a type `Details`, and then interpolate it with `$Details` in the path.
///
/// ```
/// # use serde_json::from_str;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let france = "france = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 1.0, relative_humidity = 2.0, air_temperature = 3.0 } } } }] } }";
/// # let japan = "japan = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 4.0, relative_humidity = 5.0, air_temperature = 6.0 } } } }] } }";
/// # use serde_cursor::Cursor;
/// type Details<RestOfPath> = serde_cursor::Path!(properties.timeseries[].data.instant.details + RestOfPath);
///
/// let pressure: Vec<f64> = toml::from_str::<Cursor!(france.$Details.air_pressure_at_sea_level)>(france)?.0;
/// let humidity: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.relative_humidity)>(japan)?.0;
/// let temperature: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.air_temperature)>(japan)?.0;
/// # Ok(()) }
/// ```
///
/// # Under the hood
///
/// The type returned `Cursor!` here:
///
/// ```
/// # type C =
/// serde_cursor::Cursor!(package[].dependencies: String)
/// # ;
/// ```
///
/// Is equivalent to the `Cursor` **type**, with the 2nd argument being a call of the `Path!` macro:
///
/// ```
/// # type C =
/// serde_cursor::Cursor<String, serde_cursor::Path!(package[].dependencies + serde_cursor::PathEnd)>
/// # ;
/// ```
///
/// See the [crate-level](https://docs.rs/serde_cursor/latest/serde_cursor) documentation for more info.
#[proc_macro]
#[allow(nonstandard_style)]
pub fn Path(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let cursor_path_segments = match parse_path_segments(&mut input, '+') {
        Ok(value) => value,
        Err(compile_error) => return compile_error,
    };

    let path = match path::Path::parse(&mut input) {
        Ok(ident) => ident,
        Err(compile_error) => return compile_error.into(),
    };

    // Cursor path: `Path<_, Path<_, T>>`
    build_path(
        cursor_path_segments,
        TokenStream::from_iter(path.into_tokens()),
    )
}

fn build_path(cursor_path_segments: Vec<PathSegment>, end: TokenStream) -> TokenStream {
    cursor_path_segments
        .into_iter()
        .rev()
        .fold(end, |p, segment| {
            enum GenericArgs {
                One(TokenStream),
                Two(TokenStream, TokenStream),
            }

            fn build_generic_type(name: &str, args: GenericArgs) -> TokenStream {
                let mut ts = path([ident(name)]);
                ts.extend([punct('<')]);
                match args {
                    GenericArgs::One(a) => {
                        ts.extend([TokenTree::Group(Group::new(Delimiter::Brace, a))]);
                    }
                    GenericArgs::Two(a, b) => {
                        ts.extend([TokenTree::Group(Group::new(Delimiter::Brace, a))]);
                        ts.extend([punct(',')]);
                        ts.extend([TokenTree::Group(Group::new(Delimiter::Brace, b))]);
                    }
                }
                ts.extend([punct('>')]);
                ts
            }

            let segment = match segment {
                PathSegment::Interpolated { path, dollar: _ } => {
                    // Interpolated<P>

                    let mut ts = TokenStream::from_iter(path.into_tokens());

                    // Interpolated<Path<...>>
                    //             ^
                    ts.extend([punct('<')]);

                    // Interpolated<Path<...>>
                    //              ^^^^^^^^^^^^^^^
                    ts.extend(p);

                    // Interpolated<Path<...>>
                    //                              ^
                    ts.extend([punct('>')]);

                    return ts;
                }
                PathSegment::Field { value, spans } => const_str::encode(&value, &spans),
                PathSegment::Index(index_seg) => {
                    match index_seg {
                        IndexPathSegment::RangeFull => path([ident("RangeFull")]),
                        IndexPathSegment::RangeFrom(start) => {
                            build_generic_type("RangeFrom", GenericArgs::One(start))
                        }
                        IndexPathSegment::RangeTo(last) => {
                            build_generic_type("RangeTo", GenericArgs::One(last))
                        }
                        IndexPathSegment::RangeToInclusive(last) => {
                            build_generic_type("RangeToInclusive", GenericArgs::One(last))
                        }
                        IndexPathSegment::Index(index) => {
                            build_generic_type("Index", GenericArgs::One(index))
                        }
                        IndexPathSegment::Range(start, last) => {
                            build_generic_type("Range", GenericArgs::Two(start, last))
                        }
                        IndexPathSegment::RangeInclusive(start, last) => {
                            build_generic_type("RangeInclusive", GenericArgs::Two(start, last))
                        }
                    }
                }
            };

            let mut ts = path([ident("Path")]);
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
        // Cursor!(a[0].c: bool)
        //         ^
        if !started {
            match tt {
                TokenTree::Group(g) if g.delimiter() == Delimiter::Bracket => {
                    let index = parse_index(g.span(), g.stream())?;

                    cursor_path_segments.push(PathSegment::Index(index));

                    // eat the `[]`
                    input.next();
                }
                _ => {
                    match parse_access_path_segment(input) {
                        Ok(seg) => cursor_path_segments.push(seg),
                        Err(e) => return Err(e.into()),
                    }
                }
            }

            started = true;

            continue;
        }

        match tt {
            // Path ends at a colon, if `end_token` is `:`
            //
            // Cursor!(a.b.c: bool)
            //              ^
            TokenTree::Punct(p) if p.as_char() == end_token => {
                input.next();

                break;
            }
            // An "indexing" path segment
            //
            // Cursor!(a[].c: bool)
            //          ^^
            TokenTree::Group(g) if g.delimiter() == Delimiter::Bracket => {
                let index = parse_index(g.span(), g.stream())?;

                cursor_path_segments.push(PathSegment::Index(index));

                // eat the `[]`
                input.next();
            }
            // A single path segment
            //
            // Cursor!(a[0].c: bool)
            //         ^
            TokenTree::Punct(p) if p.as_char() == '.' => {
                input.next();

                match parse_access_path_segment(input) {
                    Ok(seg) => cursor_path_segments.push(seg),
                    Err(e) => return Err(e.into()),
                }
            }
            _ => break,
        }
    }
    Ok(cursor_path_segments)
}

/// Parse the `index` path segment.
///
/// Receives contents inside the `[...]`:
///
/// ```txt
/// Cursor!(packages[].name)
///                 ^^
/// ```
///
/// Can parse the following syntax (all inside the brackets):
///
/// ```txt
/// Cursor!(message[1..7].children)
/// Cursor!(message[1..=7].children)
/// Cursor!(message[..=7].children)
/// Cursor!(message[1..].children)
/// Cursor!(message[].children)
/// Cursor!(message[1].children)
/// ```
fn parse_index(brackets: Span, index: TokenStream) -> Result<IndexPathSegment, CompileError> {
    if index.is_empty() {
        return Ok(IndexPathSegment::RangeFull);
    }

    // all of the range's tokens
    let mut index = index.into_iter().peekable();

    // (a + 4)..b
    // ^^^^^^^ tokens before the `..`
    let mut before_range = TokenStream::new();

    while let Some(tt) = index.next() {
        // Until we find the `..`, all tokens before then are the "from" part of the range
        if !matches!(&tt, TokenTree::Punct(p) if p.as_char() == '.')
            || !matches!(index.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '.')
        {
            before_range.extend([tt]);
            continue;
        }

        // a..
        //   ^
        index.next();

        // a..
        //    ^ what comes after?
        match index.next() {
            // a..=b
            Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                // (a + 4)..=b
                //           ^ tokens after iter
                let after_range = index.collect();

                let range = if before_range.is_empty() {
                    IndexPathSegment::RangeToInclusive(after_range)
                } else {
                    IndexPathSegment::RangeInclusive(before_range, after_range)
                };

                return Ok(range);
            }
            // a..4
            Some(tt) => {
                // (a + 4)..b
                //          ^ tokens after iter
                let after_range = TokenStream::from_iter([tt].into_iter().chain(index));

                let range = if before_range.is_empty() {
                    IndexPathSegment::RangeTo(after_range)
                } else {
                    IndexPathSegment::Range(before_range, after_range)
                };

                return Ok(range);
            }
            // a..
            None => {
                if before_range.is_empty() {
                    // ..
                    return Err(CompileError::new(
                        brackets,
                        "`field[..]` is not valid, use `field[]`",
                    ));
                } else {
                    // a..
                    return Ok(IndexPathSegment::RangeFrom(before_range));
                }
            }
        }
    }

    // If we get here, that means we never had a "range" at all,
    // this is just an index instead.
    //
    // foo[index]
    let index = before_range;

    Ok(IndexPathSegment::Index(index))
}

/// Represents a path segment that has something to do with indexing
#[derive(Debug)]
enum IndexPathSegment {
    /// The range-full, `[]`.
    ///
    /// ```txt
    /// Cursor!(packages[].name)
    ///                 ^^
    /// ```
    ///
    /// The `Span` is of the `[]` group.
    RangeFull,
    RangeFrom(TokenStream),
    RangeInclusive(TokenStream, TokenStream),
    Range(TokenStream, TokenStream),
    RangeToInclusive(TokenStream),
    RangeTo(TokenStream),
    /// Index into a sequence.
    ///
    /// ```txt
    /// Cursor!(packages[].dependencies[0])
    ///                                 ^
    /// ```
    Index(TokenStream),
}

/// Represents a single segment of a path.
#[derive(Debug)]
enum PathSegment {
    /// The index-all, `*`.
    ///
    /// ```txt
    /// Cursor!(packages[].name)
    ///                  ^
    /// ```
    ///
    /// The `Span` is of the `*` token.
    Index(IndexPathSegment),
    /// An interpolated path segment
    ///
    /// ```txt
    /// type Deps<T> = serde_cursor::Path!(*.dependencies.$T);
    ///
    /// Cursor!(package.$Deps[0])
    ///                 ^^^^^
    /// ```
    Interpolated {
        /// Path to the generic type itself.
        ///
        /// ```txt
        /// Cursor!(package.$Deps[0])
        ///                 ^^^^^
        /// ```
        path: path::Path,
        /// Span of the dollar.
        ///
        /// ```txt
        /// Cursor!(package.$Deps[0])
        ///                 ^
        /// ```
        #[allow(unused)]
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
}

fn parse_access_path_segment(
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

            let path = path::Path::parse(input)?;

            Ok(PathSegment::Interpolated { path, dollar })
        }
        TokenTree::Literal(lit) => {
            let span = lit.span();
            match litrs::Literal::from(lit) {
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
                _ => Err(CompileError::new(span, "unexpected token")),
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
