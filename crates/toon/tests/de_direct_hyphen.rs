#![cfg(all(feature = "serde", feature = "de_direct"))]
use serde::Deserialize;
use serde_json::Value as JsonValue;
use toon::{Options, decode_from_str};

#[derive(Debug, Deserialize, PartialEq)]
struct User {
    id: i64,
    name: String,
}
#[derive(Debug, Deserialize, PartialEq)]
struct ItemA {
    user: User,
    active: bool,
}
#[derive(Debug, Deserialize, PartialEq)]
struct WrapA {
    items: Vec<ItemA>,
}

#[test]
fn hyphen_nested_object_and_siblings() {
    let s = "items[1]:\n  - user:\n      id: 1\n      name: Ada\n    active: true\n";
    let v: WrapA = decode_from_str(s, &Options::default()).unwrap();
    assert_eq!(v.items.len(), 1);
    assert_eq!(
        v.items[0],
        ItemA {
            user: User {
                id: 1,
                name: "Ada".into()
            },
            active: true
        }
    );
}

#[derive(Debug, Deserialize, PartialEq)]
struct ItemB {
    users: Vec<User>,
    status: String,
}
#[derive(Debug, Deserialize, PartialEq)]
struct WrapB {
    items: Vec<ItemB>,
}

#[test]
fn hyphen_tabular_header_then_sibling() {
    let s = "items[1]:\n  - users[2]{id,name}:\n    - 1,Ada\n    - 2,Bob\n    status: done\n";
    let v: WrapB = decode_from_str(s, &Options::default()).unwrap();
    assert_eq!(v.items[0].users.len(), 2);
    assert_eq!(
        v.items[0].users[0],
        User {
            id: 1,
            name: "Ada".into()
        }
    );
    assert_eq!(v.items[0].status, "done");
}

#[derive(Debug, Deserialize, PartialEq)]
struct UnicodeRow {
    #[serde(rename = "")]
    empty: Option<JsonValue>,
    #[serde(rename = "\u{0085}")]
    nel: Option<JsonValue>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct UnicodeItem {
    table: Vec<UnicodeRow>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct WrapUnicode {
    items: Vec<UnicodeItem>,
}

#[test]
fn hyphen_tabular_unicode_header() {
    let header = '\u{0085}';
    let s = format!("items[1]:\n  - table[1]{{\"\",{header}}}:\n    - null,null\n");
    let v: WrapUnicode = decode_from_str(&s, &Options::default()).unwrap();
    assert_eq!(v.items.len(), 1);
    assert_eq!(v.items[0].table.len(), 1);
    let row = &v.items[0].table[0];
    assert!(row.empty.is_none());
    assert!(row.nel.is_none());
}

#[derive(Debug, Deserialize, PartialEq)]
struct ObjC {
    tags: Vec<String>,
    nums: Vec<i64>,
}
#[test]
fn inline_primitive_arrays_as_object_fields() {
    let s = "obj:\n  tags[3]: a,b,c\n  nums[2]: 1,2\n";
    #[derive(Deserialize)]
    struct WrapC {
        obj: ObjC,
    }
    let v: WrapC = decode_from_str(s, &Options::default()).unwrap();
    assert_eq!(
        v.obj.tags,
        vec![String::from("a"), String::from("b"), String::from("c")]
    );
    assert_eq!(v.obj.nums, vec![1, 2]);
}
