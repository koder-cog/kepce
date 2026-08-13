use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, FromQueryResult, Statement, DatabaseBackend, ConnectionTrait, TransactionTrait};
use shared::entities::{users, dishes, dish_aliases, sea_orm_active_enums::{UserRoleEnum, AccountStatusEnum}};
use crate::dto::admin::{CreateDishDto, UpdateDishDto, MergeDishesDto, SplitDishDto, DetachDishDto, DishModerationStatsDto, DishAliasDto};
use sea_orm::ActiveValue::Set;
use sea_orm::ActiveModelTrait;
use bcrypt::{hash, DEFAULT_COST};
use crate::config::Config;
use uuid::Uuid;

pub async fn bootstrap_admin(db: &DatabaseConnection, config: &Config) -> Result<(), anyhow::Error> {
    if let (Some(email), Some(password)) = (&config.initial_admin_email, &config.initial_admin_password) {
        let email_lower = email.to_lowercase();
        let admin_exists = users::Entity::find()
            .filter(users::Column::Email.eq(&email_lower))
            .one(db)
            .await?
            .is_some();

        if !admin_exists {
            tracing::info!("Yönetici e-postası ({}) bulunamadı. Yapılandırma bilgileriyle oluşturuluyor...", email_lower);
            // bcrypt CPU-bound'dur; async runtime'ı bloklamamak için blocking pool'a atılır.
            let password_clone = password.clone();
            let password_hash = tokio::task::spawn_blocking(move || hash(&password_clone, DEFAULT_COST))
                .await
                .map_err(|e| anyhow::anyhow!("Blocking task failed: {}", e))??;
            
            let admin = users::ActiveModel {
                id: Set(Uuid::new_v4()),
                username: Set("admin".to_string()),
                email: Set(email_lower),
                password_hash: Set(password_hash),
                role: Set(UserRoleEnum::Admin),
                account_status: Set(AccountStatusEnum::Active),
                karma_score: Set(0),
                is_verified: Set(true),
                level: Set(99),
                level_progress: Set(0),
                ..Default::default()
            };

            admin.insert(db).await?;
            tracing::info!("Admin hesabı ({}) başarıyla oluşturuldu.", email);
        } else {
            tracing::info!("Admin hesabı ({}) veritabanında zaten mevcut, bootstrap işlemi atlandı.", email_lower);
        }
    } else {
        tracing::warn!("Admin kullanıcısı yok, ancak INITIAL_ADMIN_EMAIL veya INITIAL_ADMIN_PASSWORD eksik olduğu için kurulamadı.");
    }

    Ok(())
}

pub async fn repair_null_dish_ids(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    tracing::info!("Veritabanında dish_id değeri NULL olan yemek takma adları (alias) onarılıyor...");

    // N+1 sorgu problemini önlemek ve çok daha hızlı çalışmak için tek bir SQL sorgusu (CTE) kullanılır.
    let stmt = Statement::from_string(
        db.get_database_backend(),
        r#"
        WITH new_dishes AS (
            INSERT INTO dishes (name) 
            SELECT DISTINCT name FROM dish_aliases WHERE dish_id IS NULL
            ON CONFLICT (name) DO NOTHING
            RETURNING id, name
        )
        UPDATE dish_aliases a 
        SET dish_id = d.id 
        FROM dishes d 
        WHERE a.name = d.name AND a.dish_id IS NULL;
        "#.to_string()
    );

    db.execute(stmt).await?;

    tracing::info!("Yemek eşleşme onarımı başarıyla tamamlandı.");
    Ok(())
}

pub async fn create_dish(db: &DatabaseConnection, dto: CreateDishDto) -> Result<dishes::Model, anyhow::Error> {
    let dish = dishes::ActiveModel {
        name: Set(dto.name),
        category: Set(dto.category),
        is_celiac: Set(dto.is_celiac.unwrap_or(false)),
        is_vegan: Set(dto.is_vegan.unwrap_or(false)),
        is_vegetarian: Set(dto.is_vegetarian.unwrap_or(false)),
        ..Default::default()
    };
    Ok(dish.insert(db).await?)
}

pub async fn update_dish(db: &DatabaseConnection, id: i32, dto: UpdateDishDto) -> Result<dishes::Model, anyhow::Error> {
    let mut dish: dishes::ActiveModel = dishes::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Yemek bulunamadı"))?
        .into();

    if let Some(name) = dto.name { dish.name = Set(name); }
    if let Some(category) = dto.category { dish.category = Set(Some(category)); }
    if let Some(is_celiac) = dto.is_celiac { dish.is_celiac = Set(is_celiac); }
    if let Some(is_vegan) = dto.is_vegan { dish.is_vegan = Set(is_vegan); }
    if let Some(is_vegetarian) = dto.is_vegetarian { dish.is_vegetarian = Set(is_vegetarian); }

    Ok(dish.update(db).await?)
}

pub async fn delete_dish(db: &DatabaseConnection, id: i32) -> Result<(), anyhow::Error> {
    dishes::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn merge_dishes(db: &DatabaseConnection, dto: MergeDishesDto) -> Result<(), anyhow::Error> {
    // Alias aktarımı + source silme tek transaction'da: yarım kalmış merge
    // (alias'lar gitmiş, source hâlâ duruyor) veri bütünlüğünü bozardı.
    let txn = db.begin().await?;

    // Tüm alias'ları target dish'e aktar
    dish_aliases::Entity::update_many()
        .col_expr(dish_aliases::Column::DishId, sea_orm::sea_query::Expr::value(dto.target_dish_id))
        .filter(dish_aliases::Column::DishId.eq(dto.source_dish_id))
        .exec(&txn)
        .await?;

    // Source dish'i sil
    dishes::Entity::delete_by_id(dto.source_dish_id).exec(&txn).await?;

    txn.commit().await?;
    Ok(())
}

pub async fn detach_dish(db: &DatabaseConnection, dto: DetachDishDto) -> Result<(), anyhow::Error> {
    let mut alias: dish_aliases::ActiveModel = dish_aliases::Entity::find_by_id(dto.alias_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Alias bulunamadı"))?
        .into();
    
    alias.dish_id = Set(None);
    alias.update(db).await?;
    Ok(())
}

pub async fn split_dish(db: &DatabaseConnection, dto: SplitDishDto) -> Result<dishes::Model, anyhow::Error> {
    let txn = db.begin().await?;

    // 1. Orijinal yemeği bul
    let original_dish = dishes::Entity::find_by_id(dto.dish_id)
        .one(&txn)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Yemek bulunamadı"))?;

    // 2. Delimiter'a göre böl
    let parts: Vec<String> = original_dish.name
        .split(&dto.delimiter)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() <= 1 {
        anyhow::bail!("Yemek ismi belirtilen delimiter ile bölünemedi.");
    }

    let mut new_dishes = Vec::new();
    let mut new_alias_ids = Vec::new();

    // 3. Bölünen her parça için yemek ve alias oluştur veya bul
    for part_name in &parts {
        let existing_dish = dishes::Entity::find()
            .filter(dishes::Column::Name.eq(part_name))
            .one(&txn)
            .await?;

        let dish_id = match existing_dish {
            Some(d) => {
                new_dishes.push(d.clone());
                d.id
            }
            None => {
                let new_d = dishes::ActiveModel {
                    name: Set(part_name.clone()),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
                new_dishes.push(new_d.clone());
                new_d.id
            }
        };

        let existing_alias = dish_aliases::Entity::find()
            .filter(dish_aliases::Column::Name.eq(part_name))
            .one(&txn)
            .await?;

        let alias_id = match existing_alias {
            Some(a) => {
                let mut active: dish_aliases::ActiveModel = a.into();
                active.dish_id = Set(Some(dish_id));
                let updated = active.update(&txn).await?;
                updated.id
            }
            None => {
                let new_a = dish_aliases::ActiveModel {
                    name: Set(part_name.clone()),
                    dish_id: Set(Some(dish_id)),
                    ..Default::default()
                }
                .insert(&txn)
                .await?;
                new_a.id
            }
        };
        new_alias_ids.push(alias_id);
    }

    // 4. Orijinal yemeğe bağlı tüm alias'ları bul
    let original_aliases = dish_aliases::Entity::find()
        .filter(dish_aliases::Column::DishId.eq(dto.dish_id))
        .all(&txn)
        .await?;
    let original_alias_ids: Vec<i32> = original_aliases.iter().map(|a| a.id).collect();

    // 5. Bu alias'lara bağlı menü öğelerini bul ve çoğaltarak yeni alias'lara bağla
    let referenced_menus = shared::entities::menu_dishes::Entity::find()
        .filter(shared::entities::menu_dishes::Column::DishAliasId.is_in(original_alias_ids.clone()))
        .all(&txn)
        .await?;

    for md in referenced_menus {
        let menu_id = md.menu_id;
        let order_index = md.order_index;
        let is_alternative = md.is_alternative;
        let package_name = md.package_name.clone();
        let amount = md.amount.clone();
        let calories = md.calories;

        // Orijinal menü-yemek ilişkisini sil
        let active_md: shared::entities::menu_dishes::ActiveModel = md.into();
        active_md.delete(&txn).await?;

        // Her yeni bölünen yemek için yeni menü-yemek ilişkileri ekle
        for &new_alias_id in &new_alias_ids {
            let new_md = shared::entities::menu_dishes::ActiveModel {
                menu_id: Set(menu_id),
                dish_alias_id: Set(new_alias_id),
                order_index: Set(order_index),
                is_alternative: Set(is_alternative),
                package_name: Set(package_name.clone()),
                amount: Set(amount.clone()),
                calories: Set(calories),
                ..Default::default()
            };
            new_md.insert(&txn).await?;
        }
    }

    // 6. Orijinal alias'ları ve orijinal yemeği sil
    for a in original_aliases {
        let active: dish_aliases::ActiveModel = a.into();
        active.delete(&txn).await?;
    }
    dishes::Entity::delete_by_id(dto.dish_id).exec(&txn).await?;

    txn.commit().await?;

    Ok(new_dishes.first().cloned().unwrap())
}

#[derive(FromQueryResult)]
struct DishStatsResult {
    id: i32,
    name: String,
    category: Option<String>,
    is_celiac: bool,
    is_vegan: bool,
    is_vegetarian: bool,
    usage_count: i64,
}

pub async fn get_dish_stats(db: &DatabaseConnection, search: Option<String>) -> Result<Vec<DishModerationStatsDto>, anyhow::Error> {
    let mut sql = r#"
        SELECT 
            d.id, d.name, d.category, d.is_celiac, d.is_vegan, d.is_vegetarian,
            COUNT(md.id) as usage_count
        FROM dishes d
        LEFT JOIN dish_aliases da ON d.id = da.dish_id
        LEFT JOIN menu_dishes md ON da.id = md.dish_alias_id
    "#.to_string();

    let mut values = vec![];
    if let Some(s) = search {
        if !s.trim().is_empty() {
            sql.push_str(" WHERE LOWER(d.name) LIKE LOWER($1)");
            values.push(format!("%{}%", s.trim()).into());
        }
    }
    sql.push_str(" GROUP BY d.id ORDER BY usage_count DESC");

    let stmt = Statement::from_sql_and_values(DatabaseBackend::Postgres, &sql, values);
    let results = DishStatsResult::find_by_statement(stmt).all(db).await?;

    let aliases = dish_aliases::Entity::find().all(db).await?;

    let mut dtos = Vec::new();
    for row in results {
        let mut constraints = Vec::new();
        if row.is_vegan { constraints.push("Vegan".to_string()); }
        if row.is_vegetarian { constraints.push("Vejetaryen".to_string()); }
        if row.is_celiac { constraints.push("Glutensiz".to_string()); }

        let my_aliases: Vec<DishAliasDto> = aliases.iter()
            .filter(|a| a.dish_id == Some(row.id))
            .map(|a| DishAliasDto {
                id: a.id,
                name: a.name.clone(),
            })
            .collect();

        dtos.push(DishModerationStatsDto {
            id: row.id,
            name: row.name,
            category: row.category,
            constraints,
            usage_count: row.usage_count,
            aliases: my_aliases,
        });
    }

    Ok(dtos)
}
