#![cfg(feature = "serde-compat")]
#![allow(dead_code)]

use rust_py_models::PY;

#[derive(PY)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SerdeStatus {
    AwaitingReview,
    InProgress,
}

#[derive(PY)]
#[py(export)]
#[serde(rename_all = "camelCase", default)]
struct SerdeRecord {
    user_id: u64,
    review_status: SerdeStatus,
    #[py(rename = "explicit_name")]
    original_name: String,
}

#[derive(PY)]
struct Metadata {
    request_id: u64,
}

#[derive(PY)]
struct FlattenedRecord {
    #[serde(flatten)]
    metadata: Metadata,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(default)]
    enabled: bool,
    #[serde(skip)]
    internal_note: String,
}

#[derive(PY)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum InternallyTagged {
    Ready,
    WithData {
        item_count: u64,
    },
    #[serde(skip)]
    Hidden,
}

#[derive(PY)]
#[serde(tag = "kind", content = "payload", rename_all = "kebab-case")]
enum AdjacentlyTagged {
    Ready,
    Count(u64),
    Details { item_id: u64 },
}

#[derive(PY)]
#[serde(untagged)]
enum UntaggedValue {
    Text(String),
    Details { value: u64 },
}

#[derive(PY)]
#[py(export)]
struct SerdeEnvelope {
    flattened: FlattenedRecord,
    internally_tagged: InternallyTagged,
    adjacently_tagged: AdjacentlyTagged,
    untagged: UntaggedValue,
}

#[test]
fn applies_supported_serde_rename_all_rules() {
    let declaration = SerdeRecord::export_to_string().unwrap();
    assert!(declaration.contains("userId: int"), "{declaration}");
    assert!(
        declaration.contains("reviewStatus: SerdeStatus"),
        "{declaration}"
    );
    assert!(declaration.contains("explicit_name: str"), "{declaration}");

    let status = SerdeStatus::export_to_string().unwrap();
    assert!(
        status.contains("AwaitingReview = \"AWAITING_REVIEW\""),
        "{status}"
    );
    assert!(status.contains("InProgress = \"IN_PROGRESS\""), "{status}");
}

#[test]
fn supports_field_serde_attributes_and_flattening() {
    let declaration = FlattenedRecord::export_to_string().unwrap();
    assert!(declaration.contains("request_id: int"), "{declaration}");
    assert!(declaration.contains("displayName: str"), "{declaration}");
    assert!(declaration.contains("enabled: bool"), "{declaration}");
    assert!(!declaration.contains("metadata:"), "{declaration}");
    assert!(!declaration.contains("internal_note"), "{declaration}");
}

#[test]
fn renders_internally_tagged_enums() {
    let declaration = InternallyTagged::export_to_string().unwrap();
    assert!(
        declaration
            .contains("kind: Literal[\"with_data\"] = field(init=False, default=\"with_data\")"),
        "{declaration}"
    );
    assert!(declaration.contains("itemCount: int"), "{declaration}");
    assert!(!declaration.contains("Hidden"), "{declaration}");
}

#[test]
fn renders_adjacently_tagged_enums() {
    let declaration = AdjacentlyTagged::export_to_string().unwrap();
    assert!(
        declaration.contains("kind: Literal[\"count\"] = field(init=False, default=\"count\")"),
        "{declaration}"
    );
    assert!(declaration.contains("payload: int"), "{declaration}");
    assert!(
        declaration.contains("class AdjacentlyTaggedDetailsContent"),
        "{declaration}"
    );
    assert!(
        declaration.contains("payload: AdjacentlyTaggedDetailsContent"),
        "{declaration}"
    );
}

#[test]
fn renders_untagged_payload_enums_without_discriminators() {
    let declaration = UntaggedValue::export_to_string().unwrap();
    assert!(
        declaration.contains("class UntaggedValueText"),
        "{declaration}"
    );
    assert!(!declaration.contains("Literal["), "{declaration}");
    assert!(!declaration.contains("field(init=False"), "{declaration}");
}
