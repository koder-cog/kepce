use std::sync::OnceLock;
use anyhow::Result;
use regex::Regex;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, FromQueryResult, Statement,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    pub dishes_updated: usize,
    pub dishes_merged: usize,
    pub aliases_updated: usize,
    pub aliases_merged: usize,
}

#[derive(Debug, FromQueryResult)]
struct DishRow {
    id: i32,
    name: String,
}

#[derive(Debug, FromQueryResult)]
struct AliasRow {
    id: i32,
    name: String,
    dish_id: Option<i32>,
}

#[derive(Debug, FromQueryResult)]
struct IdRow {
    id: i32,
}

/// Normalizes dish alias names for consistent database storage:
/// pads '+' symbols, collapses spaces, cleans unit dots, and resolves compound words.
pub fn normalize_alias_name(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    if s.is_empty() {
        return s;
    }

    // 1. Spacing around plus signs
    static RE_PLUS: OnceLock<Regex> = OnceLock::new();
    let re_plus = RE_PLUS.get_or_init(|| Regex::new(r"\s*\+\s*").unwrap());
    s = re_plus.replace_all(&s, " + ").to_string();

    // 2. Normalize whitespace
    static RE_SPACES: OnceLock<Regex> = OnceLock::new();
    let re_spaces = RE_SPACES.get_or_init(|| Regex::new(r"\s+").unwrap());
    s = re_spaces.replace_all(&s, " ").to_string();

    s = s.trim_start_matches(|c: char| c == '+' || c.is_whitespace())
         .trim_end_matches(|c: char| c == '+' || c.is_whitespace())
         .to_string();

    // 3. Unit dots (500 ml. -> 500 ml)
    static RE_UNIT_DOTS: OnceLock<Regex> = OnceLock::new();
    let re_unit_dots = RE_UNIT_DOTS.get_or_init(|| Regex::new(r"(?i)\b(\d+)\s*(ml|g|gr|kg|l|lt)\.").unwrap());
    s = re_unit_dots.replace_all(&s, "$1 $2").to_string();

    // 4. TDK Compound Words
    static RE_DEREOTU: OnceLock<Regex> = OnceLock::new();
    let re_dereotu = RE_DEREOTU.get_or_init(|| Regex::new(r"(?i)\bdere\s+ot(u|lu)?\b").unwrap());
    s = re_dereotu.replace_all(&s, "dereot$1").to_string();

    static RE_SEMIZOTU: OnceLock<Regex> = OnceLock::new();
    let re_semizotu = RE_SEMIZOTU.get_or_init(|| Regex::new(r"(?i)\bsemiz\s+ot(u|lu)?\b").unwrap());
    s = re_semizotu.replace_all(&s, "semizot$1").to_string();

    static RE_COREKOTU: OnceLock<Regex> = OnceLock::new();
    let re_corekotu = RE_COREKOTU.get_or_init(|| Regex::new(r"(?i)\bçöre[k]?\s+ot(u|lu)?\b").unwrap());
    s = re_corekotu.replace_all(&s, "çöreot$1").to_string();

    static RE_KURUFASULYE: OnceLock<Regex> = OnceLock::new();
    let re_kurufasulye = RE_KURUFASULYE.get_or_init(|| Regex::new(r"(?i)\bkurufasulye\b").unwrap());
    s = re_kurufasulye.replace_all(&s, "kuru fasulye").to_string();

    // 5. Clean trailing dot
    let trimmed = s.trim_end_matches('.').trim();

    // 6. Apply standard Turkish title casing
    crate::parser::normalizer::turkish_title_case(trimmed)
}

/// Scans existing dishes and dish aliases, applying normalization rules.
/// Handles collisions by repointing foreign keys and removing duplicate records.
/// Fully idempotent: running repeatedly produces 0 mutations when clean.
pub async fn reconcile_and_normalize_dishes(db: &DatabaseConnection) -> Result<ReconcileReport> {
    let mut report = ReconcileReport::default();

    // -------------------------------------------------------------
    // 1. Normalizing dishes
    // -------------------------------------------------------------
    let dishes = DishRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT id, name FROM dishes ORDER BY id ASC;",
        vec![],
    ))
    .all(db)
    .await?;

    for dish in dishes {
        let normalized = crate::parser::normalizer::normalize_food_name(&dish.name);
        if normalized == dish.name {
            continue;
        }

        // Check if a dish with the normalized name already exists
        let existing = IdRow::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id FROM dishes WHERE name = $1 AND id != $2 LIMIT 1;",
            vec![normalized.clone().into(), dish.id.into()],
        ))
        .one(db)
        .await?;

        if let Some(target) = existing {
            let target_id = target.id;
            let old_id = dish.id;

            // 1. repoint dish_aliases
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE dish_aliases SET dish_id = $1 WHERE dish_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 2. repoint comments
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE comments SET dish_id = $1 WHERE dish_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 3. repoint dish_votes (delete duplicates first)
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "DELETE FROM dish_votes WHERE dish_id = $2 AND EXISTS (
                    SELECT 1 FROM dish_votes target
                    WHERE target.dish_id = $1
                      AND target.menu_id = dish_votes.menu_id
                      AND target.user_id = dish_votes.user_id
                );",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE dish_votes SET dish_id = $1 WHERE dish_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 4. repoint user_favorites
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "DELETE FROM user_favorites WHERE dish_id = $2 AND EXISTS (
                    SELECT 1 FROM user_favorites target
                    WHERE target.dish_id = $1
                      AND target.user_id = user_favorites.user_id
                );",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE user_favorites SET dish_id = $1 WHERE dish_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 5. repoint user_pinned_dishes
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "DELETE FROM user_pinned_dishes WHERE dish_id = $2 AND EXISTS (
                    SELECT 1 FROM user_pinned_dishes target
                    WHERE target.dish_id = $1
                      AND target.user_id = user_pinned_dishes.user_id
                );",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE user_pinned_dishes SET dish_id = $1 WHERE dish_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 6. repoint dish_tags
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "DELETE FROM dish_tags WHERE dish_id = $2 AND EXISTS (
                    SELECT 1 FROM dish_tags target
                    WHERE target.dish_id = $1
                      AND target.tag_id = dish_tags.tag_id
                );",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE dish_tags SET dish_id = $1 WHERE dish_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 7. repoint dishes.parent_id
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE dishes SET parent_id = $1 WHERE parent_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 8. delete old dish
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "DELETE FROM dishes WHERE id = $1;",
                vec![old_id.into()],
            ))
            .await?;

            report.dishes_merged += 1;
        } else {
            // No target collision: simple rename
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE dishes SET name = $1 WHERE id = $2;",
                vec![normalized.into(), dish.id.into()],
            ))
            .await?;

            report.dishes_updated += 1;
        }
    }

    // -------------------------------------------------------------
    // 2. Normalizing dish_aliases
    // -------------------------------------------------------------
    let aliases = AliasRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT id, name, dish_id FROM dish_aliases ORDER BY id ASC;",
        vec![],
    ))
    .all(db)
    .await?;

    for alias in aliases {
        let normalized = normalize_alias_name(&alias.name);
        if normalized == alias.name {
            continue;
        }

        // Check if an alias with the normalized name already exists
        let existing = AliasRow::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id, name, dish_id FROM dish_aliases WHERE name = $1 AND id != $2 LIMIT 1;",
            vec![normalized.clone().into(), alias.id.into()],
        ))
        .one(db)
        .await?;

        if let Some(target) = existing {
            let target_id = target.id;
            let old_id = alias.id;

            // 1. Delete duplicate menu_dishes rows for (menu_id, package_name)
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "DELETE FROM menu_dishes WHERE dish_alias_id = $2 AND EXISTS (
                    SELECT 1 FROM menu_dishes target
                    WHERE target.dish_alias_id = $1
                      AND target.menu_id = menu_dishes.menu_id
                      AND target.package_name = menu_dishes.package_name
                );",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 2. Repoint remaining menu_dishes rows
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE menu_dishes SET dish_alias_id = $1 WHERE dish_alias_id = $2;",
                vec![target_id.into(), old_id.into()],
            ))
            .await?;

            // 3. If target has no dish_id and old alias had one, adopt it
            if target.dish_id.is_none() && alias.dish_id.is_some() {
                db.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    "UPDATE dish_aliases SET dish_id = $1 WHERE id = $2;",
                    vec![alias.dish_id.into(), target_id.into()],
                ))
                .await?;
            }

            // 4. Delete old alias
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "DELETE FROM dish_aliases WHERE id = $1;",
                vec![old_id.into()],
            ))
            .await?;

            report.aliases_merged += 1;
        } else {
            // No target collision: simple rename
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE dish_aliases SET name = $1 WHERE id = $2;",
                vec![normalized.into(), alias.id.into()],
            ))
            .await?;

            report.aliases_updated += 1;
        }
    }

    // -------------------------------------------------------------
    // 3. Canonical dish link catch-up
    // -------------------------------------------------------------
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "UPDATE dish_aliases da
         SET dish_id = d.id
         FROM dishes d
         WHERE da.dish_id IS NULL AND da.name = d.name;",
        vec![],
    ))
    .await?;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_alias_name() {
        assert_eq!(normalize_alias_name("Bal+tereyağ"), "Bal + Tereyağ");
        assert_eq!(normalize_alias_name("500 ml. su"), "500 ml Su");
        assert_eq!(normalize_alias_name("Dere Otlu Poğaça"), "Dereotlu Poğaça");
        assert_eq!(normalize_alias_name("Kurufasulye"), "Kuru Fasulye");
        assert_eq!(normalize_alias_name("Semiz Otu"), "Semizotu");
        assert_eq!(normalize_alias_name("+Tavuk Sote + Pilav+"), "Tavuk Sote + Pilav");
    }
}
