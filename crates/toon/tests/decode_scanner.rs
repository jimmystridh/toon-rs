use toon::decode::scanner::{LineKind, scan};

#[test]
fn scan_basic_object_and_list() {
    let input = "a: 1\nb:\n  - true\n  - \"x\"\n";
    let lines = scan(input);
    assert!(matches!(lines[0].kind, LineKind::KeyValue { .. }));
    assert_eq!(lines[0].indent, 0);
    if let LineKind::KeyValue { key, value } = &lines[0].kind {
        assert_eq!(*key, "a");
        assert_eq!(*value, "1");
    }
    assert!(matches!(lines[1].kind, LineKind::KeyOnly { .. }));
    assert_eq!(lines[1].indent, 0);
    assert!(matches!(lines[2].kind, LineKind::ListItem { .. }));
    assert_eq!(lines[2].indent, 2);
    assert!(matches!(lines[3].kind, LineKind::ListItem { .. }));
    assert_eq!(lines[3].indent, 2);
}
