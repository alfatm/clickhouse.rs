#[test]
fn test_flatten_fields() {
    use clickhouse::Row;
    use serde::Serialize;

    #[derive(Row, Serialize)]
    #[allow(dead_code)]
    struct MyRow {
        foo: i32,
        #[serde(flatten)]
        inner: Inner,
        baz: String,
    }

    #[derive(Row, Serialize)]
    #[allow(dead_code)]
    struct Inner {
        bar: Option<i32>,
    }

    assert_eq!(MyRow::COLUMN_NAMES, ["foo", "bar", "baz"]);
}
