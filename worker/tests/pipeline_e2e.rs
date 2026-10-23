#[test]
fn test_pipeline_smoke() {
    let db = worker::domain::models::MenuDatabase::new();
    assert!(db.is_empty());
}
