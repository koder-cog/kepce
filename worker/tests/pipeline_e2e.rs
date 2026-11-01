#[test]
fn test_pipeline_smoke() {
    let db = worker::domain::models::MenuDatabase::new();
    assert!(db.is_empty());
}

#[test]
fn test_erzincan_excel() {
    let mut db = worker::domain::models::MenuDatabase::new();
    let path = std::env::var("ERZINCAN_EXCEL_PATH").unwrap_or_default();
    if path.is_empty() || !std::path::Path::new(&path).exists() {
        return;
    }
    let diag = worker::pipeline::l0_router::excel::parse_excel_with_diagnostics(&path, &mut db)
        .expect("parse_excel failed");
    assert_eq!(diag.unparseable_dates.len(), 0);
    assert_eq!(
        db.len(),
        30,
        "Kasım ayının 30 günü eksiksiz ayrıştırılmalıdır"
    );

    // 18 Kasım: Yüzdelik kotaların (%80, %20) temizlendiğini ve alternatiflerin bölündüğünü doğrula
    let d18 = db.get("2026-11-18").expect("18 Kasım verisi bulunamadı");
    assert_eq!(
        d18.normal.dinner.len(),
        6,
        "18 Kasım akşam menüsü 6 kap olmalıdır"
    );
    let main_dishes = &d18.normal.dinner[1].alternatives;
    assert_eq!(main_dishes.len(), 2, "Ana yemek 2 alternatif içermelidir");
    assert_eq!(main_dishes[0].name, "Çıtır Tavuk");
    assert_eq!(main_dishes[0].amount.as_deref(), Some("100 g et / 200 g"));
    assert_eq!(main_dishes[1].name, "Karışık Dolma + Yoğurt");
    assert_eq!(main_dishes[1].amount.as_deref(), Some("200 g"));

    // 17 Kasım: 3 alternatifli ana yemek ve gram/et porsiyonları
    let d17 = db.get("2026-11-17").expect("17 Kasım verisi bulunamadı");
    let d17_main = &d17.normal.dinner[1].alternatives;
    assert_eq!(
        d17_main.len(),
        3,
        "17 Kasım ana yemek 3 alternatif içermelidir"
    );
    assert_eq!(d17_main[0].name, "Sulu Köfte");
    assert_eq!(d17_main[0].amount.as_deref(), Some("60 g et / 250 g"));
    assert_eq!(d17_main[1].name, "Tavuk Fajita");
    assert_eq!(d17_main[1].amount.as_deref(), Some("100 g et / 250 g"));
    assert_eq!(d17_main[2].name, "Karnabahar Kızartma");
    assert_eq!(d17_main[2].amount.as_deref(), Some("200 g"));

    // Çorba kalori aralıkları (120-160) doğrulaması
    let soup_alts = &d17.normal.dinner[0].alternatives;
    assert_eq!(soup_alts[0].calories.as_deref(), Some("120-160"));
    assert_eq!(soup_alts[1].calories.as_deref(), Some("120-160"));
}
