// This module is adapted from the `monostate` crate by `dtolnay`, MIT license.
//
// ref: https://github.com/dtolnay/monostate/blob/efb63b7ab6bfe73e7ccf20f71d17d3705cff9fcc/src/string.rs

// We encode the chars at two consecutive levels of a K-ary tree.
//
// Suppose K=3, then strings "", "a", "ab", "abc", … would be encoded to:
//     ()
//     (a)
//     (a, b)
//     (a, b, c)
//     (a, b, (c, d))
//     (a, b, (c, d, e))
//     (a, (b, c), (d, e, f))
//     (a, (b, c, d), (e, f, g))
//     ((a, b), (c, d, e), (f, g, h))
//     ((a, b, c), (d, e, f), (g, h, i))
//     ((a, b, c), (d, e, f), (g, h, (i, j)))
//     ((a, b, c), (d, e, f), (g, h, (i, j, k)))
//     ((a, b, c), (d, e, f), (g, (h, i), (j, k l)))
//
// That last one in tree form is:
//           ╷
//      ┌────┴┬──────┐
//     ┌┴┬─┐ ┌┴┬─┐ ┌─┴┬───┐
//     a b c d e f g ┌┴┐ ┌┴┬─┐
//                   h i j k l
//
// The value of K is however many nested implementations of `StringBuffer` there
// are for a maximum tuple length of K. `StringBuffer` is implemented for tuples of
// `StringBuffer`s up to length 6, so K=6.
//
// (impl StringBuffer, impl StringBuffer, impl StringBuffer) implements StringBuffer,
// and so on for tuples of size 1, 2, 3, 4, 5, and 6. So they can be nested arbitrarily
// to create arbitrarily large strings.

const K: usize = 6;

use proc_macro::Delimiter;
use proc_macro::Group;
use proc_macro::Literal;
use proc_macro::Span;
use proc_macro::TokenStream;
use proc_macro::TokenTree;

use crate::ident;
use crate::path;
use crate::punct;

/// Encodes a string literal into a nested type-level representation:
/// `FieldName<(StrLen<N>, ( ... nested tuples of characters ... ))>`
pub fn encode(value: &str, spans: &[Span]) -> TokenStream {
    // encoding of empty string
    if value.is_empty() {
        let mut ts = path([ident("FieldName")]);
        ts.extend([punct('<')]);

        let mut tuple = TokenStream::new();
        // Generates StrLen<0>
        let mut str_len = path([ident("StrLen")]);
        str_len.extend([punct('<')]);
        str_len.extend(Some(TokenTree::Literal(Literal::usize_unsuffixed(0))));
        str_len.extend([punct('>')]);

        tuple.extend(str_len);
        tuple.extend([punct(',')]);
        // Represents an empty unit tuple () for the content
        tuple.extend(Some(TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::new(),
        ))));

        ts.extend(Some(TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            tuple,
        ))));
        ts.extend([punct(',')]);
        ts.extend([ident("false")]);
        ts.extend([punct('>')]);
        return ts;
    }

    // 1/3: create leaf nodes

    // convert every character into its type-level wrapper, e.g., C1<'a'>, C2<'β'>, etc.
    let mut nodes: Vec<TokenStream> = value
        .chars()
        .map(|ch| {
            let len = ch.len_utf8();
            // Prefix corresponds to the byte-length of the UTF-8 character
            let prefix = match len {
                1 => "C1",
                2 => "C2",
                3 => "C3",
                4 => "C4",
                _ => unreachable!("UTF-8 chars are max 4 bytes"),
            };
            let mut ts = path([ident(prefix)]);
            ts.extend([punct('<')]);
            let lit = Literal::character(ch);
            ts.extend(Some(TokenTree::Literal(lit)));
            ts.extend([punct('>')]);
            ts
        })
        .collect();

    // 2/3: K-ary tree construction

    // we group nodes from the end of the list into tuples of size K.
    // this reduces the number of top-level nodes until only one remains.

    // find the largest power of K that is strictly less than our node count.
    let mut pow = 1;
    while pow * K < nodes.len() {
        pow *= K;
    }

    while nodes.len() > 1 {
        // how many nodes exist beyond the current balanced "floor"
        let overage = nodes.len() - pow;

        // we need to group these overage nodes into tuples.
        // each tuple consumes K existing nodes and replaces them with 1 new tuple node.
        // math: ceil(overage / (K - 1))
        let num_tuple_nodes = (overage + K - 2) / (K - 1);
        let total_to_process = num_tuple_nodes + overage;
        let read_start = nodes.len() - total_to_process;

        // keep the "prefix" nodes that aren't being grouped in this pass.
        let mut new_nodes = nodes[0..read_start].to_vec();
        let to_be_grouped = &nodes[read_start..];

        // chunk the selected nodes into groups of K and wrap them in parentheses to make tuples.
        for chunk in to_be_grouped.chunks(K) {
            let mut tuple_stream = TokenStream::new();
            for (i, node) in chunk.iter().enumerate() {
                if i > 0 {
                    tuple_stream.extend([punct(',')]);
                }
                tuple_stream.extend(node.clone());
            }
            new_nodes.push(TokenStream::from(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                tuple_stream,
            ))));
        }

        nodes = new_nodes;
        // move up one level in the tree
        pow /= K;
    }

    // 3/3: assembly

    // wrap the final tree root in FieldName<(StrLen<L>, Root)>
    let mut ts = path([ident("FieldName")]);
    ts.extend([punct('<')]);

    let mut inner_tuple = TokenStream::new();
    let mut slen = path([ident("StrLen")]);
    slen.extend([punct('<')]);
    slen.extend(Some(TokenTree::Literal(Literal::usize_unsuffixed(
        value.len(),
    ))));
    slen.extend([punct('>')]);

    inner_tuple.extend(slen);
    inner_tuple.extend([punct(',')]);

    // the single remaining node, the root of our tuple tree
    inner_tuple.extend(nodes.remove(0));

    ts.extend(Some(TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        inner_tuple,
    ))));

    // See the comment on `FieldName` for WHY we create a bunch of garbage
    // tokens that aren't used for anything. (tldr: syntax-highlighting in IDEs)
    //
    // ```txt
    // FieldName<..., { ["", "", ""]; false }>
    //                   ^^ "dev"
    //                       ^^ "-"
    //                           ^^ "dependencies"
    // ```
    {
        ts.extend([punct(',')]);

        // { ["", "", ""]; false }
        //    ^^^^^^^^^^
        let strings: TokenStream = spans
            .iter()
            .flat_map(|span| {
                // { ["", "", ""]; false }
                //    ^^
                let mut lit = TokenTree::Literal(Literal::string(""));
                lit.set_span(*span);

                [lit, punct(',')]
            })
            .collect();

        // { ["", "", ""]; false }
        // ^^^^^^^^^^^^^^^^^^^^^^^
        ts.extend([TokenTree::Group(Group::new(
            Delimiter::Brace,
            TokenStream::from_iter([
                TokenTree::Group(Group::new(Delimiter::Bracket, strings)),
                punct(';'),
                ident("false"),
            ]),
        ))]);
    }

    ts.extend([punct('>')]);

    ts
}
