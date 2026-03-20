use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

const K: usize = 6;

#[proc_macro]
#[allow(nonstandard_style)]
pub fn Cursor(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let mut path_segments = Vec::new();

    let mut started = false;

    // Parse path segments
    //
    // Cursor!(a.0.c, bool)
    //         ^^^^^
    while let Some(tt) = input.peek() {
        // the "." is not requires for the first path
        //
        // Cursor!(a.0.c, bool)
        //         ^
        if !started {
            parse_path_segment(&mut input, &mut path_segments);
            started = true;
            continue;
        }

        match tt {
            // Path ends at a comma
            //
            // Cursor!(a.b.c, bool)
            //              ^
            TokenTree::Punct(p) if p.as_char() == ',' => {
                input.next();

                break;
            }
            // A single path segment
            //
            // Cursor!(a.0.c, bool)
            //          ^^
            TokenTree::Punct(p) if p.as_char() == '.' => {
                input.next();

                parse_path_segment(&mut input, &mut path_segments);
            }
            _ => break,
        }
    }

    // These tokens make up the actual Type.
    //
    // Cursor!(a.0.c, HashMap<&str, &str>)
    //                ^^^^^^^^^^^^^^^^^^^
    let type_tokens: TokenStream = if input.peek().is_none() {
        TokenStream::from_iter([TokenTree::Punct(Punct::new('_', Spacing::Alone))])
    } else {
        input.collect()
    };

    let mut path_ts = mk_path(vec![mk_ident("serde_cursor"), mk_ident("Nil")]);

    for segment in path_segments.into_iter().rev() {
        let mut cons = mk_path(vec![mk_ident("serde_cursor"), mk_ident("Cons")]);
        let mut args = TokenStream::new();
        args.extend(segment.to_tokens());
        args.push_punct(',', Spacing::Alone);
        args.extend(path_ts);

        cons.push_punct('<', Spacing::Alone);
        cons.extend(args);
        cons.push_punct('>', Spacing::Alone);
        path_ts = cons;
    }

    let mut cursor = mk_path(vec![mk_ident("serde_cursor"), mk_ident("Cursor")]);
    cursor.push_punct('<', Spacing::Alone);
    cursor.extend(type_tokens);
    cursor.push_punct(',', Spacing::Alone);
    cursor.extend(path_ts);
    cursor.push_punct('>', Spacing::Alone);

    cursor
}

enum PathSegment {
    Field(String, Span),
    Index(u128, Span),
}

impl PathSegment {
    fn to_tokens(&self) -> TokenStream {
        match self {
            PathSegment::Field(field, span) => encode_str(field, *span),
            PathSegment::Index(index, span) => {
                let mut ts = mk_path(vec![mk_ident("serde_cursor"), mk_ident("Index")]);
                ts.push_punct('<', Spacing::Alone);
                let mut lit = Literal::u128_unsuffixed(*index);
                lit.set_span(*span);
                ts.extend(Some(TokenTree::Literal(lit)));
                ts.push_punct('>', Spacing::Alone);
                ts
            }
        }
    }
}

fn parse_path_segment(
    input: &mut std::iter::Peekable<proc_macro::token_stream::IntoIter>,
    path_segments: &mut Vec<PathSegment>,
) {
    match input.peek().unwrap() {
        // Identifier fields
        //
        // Cursor!(a.b.c, bool)
        //         ^
        TokenTree::Ident(_) => {
            let Some(TokenTree::Ident(field)) = input.next() else {
                unreachable!()
            };
            path_segments.push(PathSegment::Field(field.to_string(), field.span()));
        }
        TokenTree::Literal(lit) => {
            match litrs::Literal::from(lit) {
                // Integer index
                //
                // Cursor!(a.0.c, bool)
                //           ^
                litrs::Literal::Integer(index) => {
                    path_segments.push(PathSegment::Index(
                        index.value::<u128>().unwrap(),
                        lit.span(),
                    ));
                    input.next();
                }
                // Integer index
                //
                // Cursor!(a."hello world".c, bool)
                //           ^^^^^^^^^^^^^
                litrs::Literal::String(field) => {
                    path_segments.push(PathSegment::Field(field.value().to_string(), lit.span()));
                    input.next();
                }
                _ => panic!(),
            };
        }
        _ => panic!(),
    }
}

/// Implements the Monostate K-ary tree logic for strings.
fn encode_str(value: &str, span: Span) -> TokenStream {
    if value.is_empty() {
        let mut ts = mk_path(vec![mk_ident("serde_cursor"), mk_ident("FieldName")]);
        ts.push_punct('<', Spacing::Alone);

        let mut tuple = TokenStream::new();
        let mut slen = mk_path(vec![mk_ident("serde_cursor"), mk_ident("StrLen")]);
        slen.push_punct('<', Spacing::Alone);
        slen.extend(Some(TokenTree::Literal(Literal::usize_unsuffixed(0))));
        slen.push_punct('>', Spacing::Alone);

        tuple.extend(slen);
        tuple.push_punct(',', Spacing::Alone);
        tuple.extend(Some(TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::new(),
        ))));

        ts.extend(Some(TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            tuple,
        ))));
        ts.push_punct('>', Spacing::Alone);
        return ts;
    }

    let mut nodes: Vec<TokenStream> = value
        .chars()
        .map(|ch| {
            let len = ch.len_utf8();
            let prefix = match len {
                1 => "C1",
                2 => "C2",
                3 => "C3",
                4 => "C4",
                _ => unreachable!("unicode scalar value (char) is a u32, so length is 4 bytes max"),
            };
            let mut ts = mk_path(vec![mk_ident("serde_cursor"), mk_ident(prefix)]);
            ts.push_punct('<', Spacing::Alone);
            let mut lit = Literal::character(ch);
            lit.set_span(span);
            ts.extend(Some(TokenTree::Literal(lit)));
            ts.push_punct('>', Spacing::Alone);
            ts
        })
        .collect();

    let mut pow = 1;
    while pow * K < nodes.len() {
        pow *= K;
    }

    while nodes.len() > 1 {
        let overage = nodes.len() - pow;
        let num_tuple_nodes = (overage + K - 2) / (K - 1);
        let remainder = num_tuple_nodes + overage;
        let read_start = nodes.len() - remainder;

        let mut new_nodes = nodes[..read_start].to_vec();
        let to_be_grouped = &nodes[read_start..];

        for chunk in to_be_grouped.chunks(K) {
            let mut tuple_stream = TokenStream::new();
            for (i, node) in chunk.iter().enumerate() {
                if i > 0 {
                    tuple_stream.push_punct(',', Spacing::Alone);
                }
                tuple_stream.extend(node.clone());
            }
            new_nodes.push(TokenStream::from(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                tuple_stream,
            ))));
        }

        nodes = new_nodes;
        pow /= K;
    }

    let mut ts = mk_path(vec![mk_ident("serde_cursor"), mk_ident("FieldName")]);
    ts.push_punct('<', Spacing::Alone);

    let mut inner_tuple = TokenStream::new();
    let mut slen = mk_path(vec![mk_ident("serde_cursor"), mk_ident("StrLen")]);
    slen.push_punct('<', Spacing::Alone);
    slen.extend(Some(TokenTree::Literal(Literal::usize_unsuffixed(
        value.len(),
    ))));
    slen.push_punct('>', Spacing::Alone);

    inner_tuple.extend(slen);
    inner_tuple.push_punct(',', Spacing::Alone);
    inner_tuple.extend(nodes.remove(0));

    ts.extend(Some(TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        inner_tuple,
    ))));
    ts.push_punct('>', Spacing::Alone);
    ts
}

fn mk_ident(name: &str) -> TokenTree {
    TokenTree::Ident(Ident::new(name, Span::call_site()))
}

fn mk_path(parts: Vec<TokenTree>) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.push_punct(':', Spacing::Joint);
    ts.push_punct(':', Spacing::Alone);
    for (i, part) in parts.into_iter().enumerate() {
        if i > 0 {
            ts.push_punct(':', Spacing::Joint);
            ts.push_punct(':', Spacing::Alone);
        }
        ts.extend(Some(part));
    }
    ts
}

trait TokenStreamExt {
    fn push_punct(&mut self, ch: char, spacing: Spacing);
}

impl TokenStreamExt for TokenStream {
    fn push_punct(&mut self, ch: char, spacing: Spacing) {
        self.extend([TokenTree::Punct(Punct::new(ch, spacing))]);
    }
}
