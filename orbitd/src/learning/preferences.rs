use anyhow::Result;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Preference management service
pub struct PreferenceService {
    db: SqlitePool,
}

impl PreferenceService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Set a preference
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO preferences (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get a preference
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let result = sqlx::query_scalar::<_, String>(
            "SELECT value FROM preferences WHERE key = ?1"
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await?;

        Ok(result)
    }

    /// Get all preferences
    pub async fn get_all(&self) -> Result<HashMap<String, String>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM preferences"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().collect())
    }

    /// Delete a preference
    pub async fn delete(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM preferences WHERE key = ?1")
            .bind(key)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_db() -> Result<SqlitePool> {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;

        sqlx::query(include_str!("../../migrations/002_learning_system.sql"))
            .execute(&pool)
            .await?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_set_get_preference() {
        let pool = create_test_db().await.unwrap();
        let service = PreferenceService::new(pool);

        service.set("test_key", "test_value").await.unwrap();
        let value = service.get("test_key").await.unwrap();

        assert_eq!(value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_preference() {
        let pool = create_test_db().await.unwrap();
        let service = PreferenceService::new(pool);

        let value = service.get("nonexistent").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_update_preference() {
        let pool = create_test_db().await.unwrap();
        let service = PreferenceService::new(pool);

        service.set("key", "value1").await.unwrap();
        service.set("key", "value2").await.unwrap();

        let value = service.get("key").await.unwrap();
        assert_eq!(value, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_get_all_preferences() {
        let pool = create_test_db().await.unwrap();
        let service = PreferenceService::new(pool);

        service.set("key1", "value1").await.unwrap();
        service.set("key2", "value2").await.unwrap();

        let all = service.get_all().await.unwrap();
        assert!(all.contains_key("key1"));
        assert!(all.contains_key("key2"));
    }

    #[tokio::test]
    async fn test_delete_preference() {
        let pool = create_test_db().await.unwrap();
        let service = PreferenceService::new(pool);

        service.set("to_delete", "value").await.unwrap();
        service.delete("to_delete").await.unwrap();

        let value = service.get("to_delete").await.unwrap();
        assert_eq!(value, None);
    }
}
