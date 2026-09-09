use rangeset::set::RangeSet;
use spansy::{
    Span,
    http::{BodyContent, parse_response},
    json::{self, JsonValue, JsonVisit, Redacted},
};

#[test]
fn redacted_values_preserve_spans_and_disclosed_fields() {
    let source = br#"{"private":***,"account":{"country":"GB","flags":[****,true,**]}}"#;
    let doc = json::parse(source).unwrap();
    let Some(JsonValue::Redacted(value)) = doc.get("private") else {
        panic!("expected a redacted value");
    };
    assert_eq!(value.view().indices(), &RangeSet::from(11usize..14));
    assert_eq!(value.data(), "***");
    assert!(doc.get("private.child").is_none());
    assert_eq!(doc.get("account.country").unwrap(), "GB");
    assert_eq!(doc.get("account.flags.1").unwrap(), "true");
    assert!(matches!(
        doc.get("account.flags.0"),
        Some(JsonValue::Redacted(_))
    ));
    assert!(matches!(
        doc.get("account.flags.2"),
        Some(JsonValue::Redacted(_))
    ));
}

#[test]
fn quoted_stars_remain_strings() {
    let doc = json::parse(br#"{"*":"***","hidden":*}"#).unwrap();
    assert!(matches!(doc.get("*"), Some(JsonValue::String(_))));
    assert!(matches!(doc.get("hidden"), Some(JsonValue::Redacted(_))));
    assert!(matches!(
        json::parse(b"***").unwrap().root,
        JsonValue::Redacted(_)
    ));
}

#[test]
fn rejects_incomplete_tokens_and_missing_structure() {
    for source in [
        r#"{"value":1**}"#,
        r#"{"value":**1}"#,
        r#"{"value":tr**}"#,
        r#"{"value":* *}"#,
        r#"{***:42}"#,
        r#"{"value":*** "other":42}"#,
        r#"[***,]"#,
        r#"{"value":***"#,
    ] {
        assert!(json::parse(source.as_bytes()).is_err(), "{source}");
    }
}

#[test]
fn visits_redacted_values() {
    #[derive(Default)]
    struct Counter(usize);

    impl<'a> JsonVisit<&'a [u8]> for Counter {
        fn visit_redacted(&mut self, _: &Redacted<&'a [u8]>) {
            self.0 += 1;
        }
    }

    let source: &[u8] = br#"{"a":***,"b":[*,"***",{"c":**}]}"#;
    let doc = json::parse(source).unwrap();
    let mut counter = Counter::default();
    counter.visit_value(&doc.root);
    assert_eq!(counter.0, 3);
}

#[test]
fn parses_redacted_chunked_json_with_original_transcript_indices() {
    let chunks: [&[u8]; 2] = [br#"{"private":**"#, br#"**,"country":"GB"}"#];
    let mut source =
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n"
            .to_vec();
    let mut starts = Vec::new();
    for chunk in chunks {
        source.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
        starts.push(source.len());
        source.extend_from_slice(chunk);
        source.extend_from_slice(b"\r\n");
    }
    source.extend_from_slice(b"0\r\n\r\n");

    let response = parse_response(source.as_slice()).unwrap();
    let BodyContent::Json(doc) = response.body.unwrap().content else {
        panic!("expected JSON body");
    };
    let Some(JsonValue::Redacted(value)) = doc.get("private") else {
        panic!("expected a redacted value");
    };
    let expected = RangeSet::from([starts[0] + 11..starts[0] + 13, starts[1]..starts[1] + 2]);
    assert_eq!(value.view().indices(), &expected);
    assert_eq!(value.data(), "****");
    let country = doc.get("country").unwrap();
    assert_eq!(country, "GB");
    assert_eq!(country.view().offset(), starts[1] + 14);
}
