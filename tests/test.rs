use serde_cursor::const_str::{C1, StrLen};
use serde_cursor_impl::Cursor;

use std::collections::HashMap;

#[test]
fn lol() {
    let value = serde_json::json!({
        "a": {
            "hello world": {
                "c": [false, false, true, false]
            }
        }
    });

    // let x = serde_json::from_value::<Cursor!(a."b lol".c, HashMap<&str, &str>)>(value)
    //     .unwrap()
    //     .value;

    // type Lol<'a> = ::serde_cursor::Cursor<
    //     HashMap<&'a str, &'a str>,
    //     ::serde_cursor::Cons<
    //         ::serde_cursor::FieldName<(::serde_cursor::StrLen<1>, ::serde_cursor::C1<'a'>)>,
    //         ::serde_cursor::Cons<
    //             ::serde_cursor::FieldName<(
    //                 ::serde_cursor::StrLen<7>,
    //                 (
    //                     ::serde_cursor::C1<'"'>,
    //                     ::serde_cursor::C1<'b'>,
    //                     ::serde_cursor::C1<' '>,
    //                     ::serde_cursor::C1<'l'>,
    //                     ::serde_cursor::C1<'o'>,
    //                     (::serde_cursor::C1<'l'>, ::serde_cursor::C1<'"'>),
    //                 ),
    //             )>,
    //             ::serde_cursor::Cons<
    //                 ::serde_cursor::FieldName<(::serde_cursor::StrLen<1>, ::serde_cursor::C1<'c'>)>,
    //                 ::serde_cursor::Nil,
    //             >,
    //         >,
    //     >,
    // >;

    let x = serde_json::from_value::<Cursor!(a."hello world".c.2, bool)>(value)
        .unwrap()
        .value;

    // let x = serde_json::from_value::<
    //     Cursor<
    //         bool, // The target type D
    //         Cons<
    //             FieldName<(StrLen<1>, C1<'a'>)>, // Segment 1
    //             Cons<
    //                 FieldName<(StrLen<1>, C1<'b'>)>, // Segment 2
    //                 Cons<
    //                     FieldName<(StrLen<1>, C1<'c'>)>, // Segment 3
    //                     Cons<
    //                         Index<2>, // Segment 4
    //                         Nil,
    //                     >,
    //                 >,
    //             >,
    //         >,
    //     >,
    // >(value)
    // .unwrap()
    // .value;

    dbg!(x);

    // let x = serde_json::from_value::<Cursor!("a"."b"."c", bool)>(value);
}
