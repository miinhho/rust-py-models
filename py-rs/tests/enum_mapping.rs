#![allow(dead_code)]

use py_rs::PY;

#[derive(PY)]
#[py(export, export_to = "auth/Event.py")]
enum Event {
    Login { user_id: i32 },
    Logout { user_id: i32 },
}

#[test]
fn payload_variants_are_dataclasses_and_union_alias() {
    let code = Event::export_to_string().unwrap();
    assert!(code.contains("class EventLogin:"));
    assert!(code.contains("class EventLogout:"));
    assert!(!code.contains("kind:"));
    assert_eq!(code.matches("    user_id: int").count(), 2);
    assert!(code.contains("Event: TypeAlias = EventLogin | EventLogout"));
}
