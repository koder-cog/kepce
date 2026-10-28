//! Şehir listesi, aktif menü mevcudiyeti ve coğrafi konum çözümleme servisi.
use sea_orm::*;
use shared::entities::cities;
use std::collections::HashSet;

#[derive(Debug)]
pub enum CityError {
    DatabaseError(DbErr),
}

impl std::fmt::Display for CityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CityError::DatabaseError(err) => write!(f, "Veritabanı hatası: {}", err),
        }
    }
}

pub struct CityService;

impl CityService {
    /// Aktif (onaylanmış menüsü olan) şehirleri getirir, celiac durumlarını ve son menü tarihlerini hesaplar.
    pub async fn get_active_cities(
        db: &DatabaseConnection,
    ) -> Result<Vec<(cities::Model, bool, Option<String>)>, CityError> {
        #[derive(sea_orm::FromQueryResult)]
        struct CityLatestMenuResult {
            city_id: i32,
            latest_menu_date: Option<String>,
        }

        // 1. Onaylı menüsü ve yemeği olan şehirlerin ID'lerini ve en son menü tarihlerini çek
        let city_latest_res = CityLatestMenuResult::find_by_statement(
            sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                SELECT m.city_id, TO_CHAR(LEAST(MAX(m.serve_date), CURRENT_DATE), 'YYYY-MM-DD') AS latest_menu_date
                FROM menus m
                INNER JOIN menu_dishes md ON m.id = md.menu_id
                WHERE m.status = 'approved'
                GROUP BY m.city_id
                "#.to_string(),
            ),
        )
        .all(db)
        .await
        .map_err(CityError::DatabaseError)?;

        if city_latest_res.is_empty() {
            return Ok(vec![]);
        }

        let latest_dates: std::collections::HashMap<i32, Option<String>> = city_latest_res
            .into_iter()
            .map(|row| (row.city_id, row.latest_menu_date))
            .collect();

        let menu_cities: Vec<i32> = latest_dates.keys().copied().collect();

        #[derive(sea_orm::FromQueryResult)]
        struct CeliacCityResult {
            city_id: i32,
        }

        // 2. Geçmişinde en az 1 tane çölyak (is_celiac=true veya package_name = ÇÖLYAK) yemeği olan şehirleri bul
        let celiac_cities_res = CeliacCityResult::find_by_statement(
            sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                r#"
                SELECT DISTINCT m.city_id
                FROM menus m
                INNER JOIN menu_dishes md ON m.id = md.menu_id
                LEFT JOIN dish_aliases da ON md.dish_alias_id = da.id
                LEFT JOIN dishes d ON da.dish_id = d.id
                WHERE d.is_celiac = true OR md.package_name ILIKE '%ÇÖLYAK%' OR md.package_name ILIKE '%COLYAK%'
                "#.to_string()
            )
        ).all(db).await.map_err(CityError::DatabaseError)?;

        let celiac_cities: HashSet<i32> = celiac_cities_res
            .into_iter()
            .map(|row| row.city_id)
            .collect();

        tracing::info!("Celiac cities: {:?}", celiac_cities);

        // 3. İlgili şehirleri is_in ile toplu olarak çek
        let mut active_cities = cities::Entity::find()
            .filter(cities::Column::Id.is_in(menu_cities))
            .all(db)
            .await
            .map_err(CityError::DatabaseError)?;

        // 4. CPU-bound sıralama işlemini (Türkçe alfabetik sıralama) asenkron akışı bloklamamak için spawn_blocking içine alıyoruz
        let sorted_cities = tokio::task::spawn_blocking(move || {
            active_cities.sort_by(|a, b| turkish_cmp(&a.name, &b.name));
            active_cities
        })
        .await
        .unwrap_or_default();

        let result = sorted_cities
            .into_iter()
            .map(|c| {
                let has_celiac = celiac_cities.contains(&c.id);
                let latest_menu_date = latest_dates.get(&c.id).cloned().flatten();
                (c, has_celiac, latest_menu_date)
            })
            .collect();

        Ok(result)
    }
}

// Türkçe karakterleri dikkate alan sıralama fonksiyonu
fn turkish_alphabet_index(c: char) -> usize {
    match c {
        'A' | 'a' => 1,
        'B' | 'b' => 2,
        'C' | 'c' => 3,
        'Ç' | 'ç' => 4,
        'D' | 'd' => 5,
        'E' | 'e' => 6,
        'F' | 'f' => 7,
        'G' | 'g' => 8,
        'Ğ' | 'ğ' => 9,
        'H' | 'h' => 10,
        'I' | 'ı' => 11,
        'İ' | 'i' => 12,
        'J' | 'j' => 13,
        'K' | 'k' => 14,
        'L' | 'l' => 15,
        'M' | 'm' => 16,
        'N' | 'n' => 17,
        'O' | 'o' => 18,
        'Ö' | 'ö' => 19,
        'P' | 'p' => 20,
        'R' | 'r' => 21,
        'S' | 's' => 22,
        'Ş' | 'ş' => 23,
        'T' | 't' => 24,
        'U' | 'u' => 25,
        'Ü' | 'ü' => 26,
        'V' | 'v' => 27,
        'Y' | 'y' => 28,
        'Z' | 'z' => 29,
        _ => 0,
    }
}

fn turkish_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars();
    let mut b_chars = b.chars();

    loop {
        match (a_chars.next(), b_chars.next()) {
            (Some(c1), Some(c2)) => {
                let idx1 = turkish_alphabet_index(c1);
                let idx2 = turkish_alphabet_index(c2);

                if idx1 != idx2 {
                    if idx1 == 0 || idx2 == 0 {
                        return c1
                            .to_lowercase()
                            .to_string()
                            .cmp(&c2.to_lowercase().to_string());
                    }
                    return idx1.cmp(&idx2);
                } else if idx1 != 0 {
                    continue;
                } else {
                    if c1.to_lowercase().to_string() != c2.to_lowercase().to_string() {
                        return c1
                            .to_lowercase()
                            .to_string()
                            .cmp(&c2.to_lowercase().to_string());
                    }
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (None, None) => return std::cmp::Ordering::Equal,
        }
    }
}
