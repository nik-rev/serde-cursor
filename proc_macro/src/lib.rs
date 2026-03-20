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

#[proc_macro]
#[allow(nonstandard_style)]
pub fn Cursor(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    // every path segment individually
    //
    // Cursor!(a.0.c: bool)
    //         ^ ^ ^
    let mut path_segments = Vec::new();

    // this is needed to know if we should expect a "." before
    // the first path segment, or not
    let mut started = false;

    // Parse path segments
    //
    // Cursor!(a.0.c: bool)
    //         ^^^^^
    while let Some(tt) = input.peek() {
        // the "." is not required for the first path
        //
        // Cursor!(a.0.c: bool)
        //         ^
        if !started {
            match parse_path_segment(&mut input) {
                Ok(seg) => path_segments.push(seg),
                Err(e) => return e.into(),
            }

            started = true;

            continue;
        }

        match tt {
            // Path ends at a colon
            //
            // Cursor!(a.b.c: bool)
            //               ^
            TokenTree::Punct(p) if p.as_char() == ':' => {
                input.next();

                break;
            }
            // A single path segment
            //
            // Cursor!(a.0.c: bool)
            //          ^^
            TokenTree::Punct(p) if p.as_char() == '.' => {
                input.next();

                match parse_path_segment(&mut input) {
                    Ok(seg) => path_segments.push(seg),
                    Err(e) => return e.into(),
                }
            }
            _ => break,
        }
    }

    // These tokens make up the actual Type.
    //
    // Cursor!(a.0.c: HashMap<&str, &str>)
    //                 ^^^^^^^^^^^^^^^^^^^
    let type_tokens: TokenStream = if input.peek().is_none() {
        TokenStream::from_iter([ident("_")])
    } else {
        input.collect()
    };

    // Type path: `Cons<_, Cons<_, Nil>>`
    let path_gen = path_segments
        .into_iter()
        .rev()
        .fold(path([ident("Nil")]), |p, segment| {
            let mut ts = path([ident("Cons")]);
            ts.extend([punct('<')]);
            ts.extend(segment.to_tokens());
            ts.extend([punct(',')]);
            ts.extend(p);
            ts.extend([punct('>')]);
            ts
        });

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
    ts.extend(path_gen);
    ts.extend([punct('>')]);

    ts
}

enum PathSegment {
    Wildcard(Span),
    Field(String, Span),
    Index(u128, Span),
}

impl PathSegment {
    fn to_tokens(&self) -> TokenStream {
        match self {
            PathSegment::Field(field, span) => const_str::encode(field, *span),
            PathSegment::Index(index, span) => {
                let mut ts = path([ident("Index")]);
                ts.extend([punct('<')]);
                let mut lit = Literal::u128_unsuffixed(*index);
                lit.set_span(*span);
                ts.extend(Some(TokenTree::Literal(lit)));
                ts.extend([punct('>')]);
                ts
            }
            PathSegment::Wildcard(_span) => path([ident("Wildcard")]),
        }
    }
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
        // Cursor!(a.b.c: bool)
        //         ^
        TokenTree::Ident(field) => {
            let segment = PathSegment::Field(field.to_string(), field.span());
            input.next();
            Ok(segment)
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
                    Ok(PathSegment::Index(val, span))
                }
                // Integer index
                //
                // Cursor!(a."hello world".c: bool)
                //           ^^^^^^^^^^^^^
                litrs::Literal::String(field) => {
                    let val = field.value().to_string();
                    input.next();
                    Ok(PathSegment::Field(val, span))
                }
                _ => {
                    Err(CompileError::new(
                        span,
                        "expected identifier, '*', integer, or string",
                    ))
                }
            }
        }
        _ => {
            Err(CompileError::new(
                tt.span(),
                "unexpected token in path segment",
            ))
        }
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
