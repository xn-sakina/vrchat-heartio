// Database management for HeartIO
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePool, Row};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HeartRateRecord {
    pub id: i64,
    pub bpm: i32,
    pub created_at: DateTime<Utc>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Create cache directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .context("Failed to create cache directory")?;
        }

        let database_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&database_url).await
            .context("Failed to connect to SQLite database")?;

        let db = Self { pool };
        db.init_tables().await?;
        
        tracing::info!("Database initialized at {}", db_path.display());
        Ok(db)
    }

    /// Get the path to the database file
    fn get_db_path() -> Result<PathBuf> {
        let exe_path = std::env::current_exe()
            .context("Failed to get current executable path")?;
        let exe_dir = exe_path.parent()
            .context("Failed to get executable directory")?;
        let cache_dir = exe_dir.join("cache");
        Ok(cache_dir.join("data.sqlite"))
    }

    /// Initialize database tables
    async fn init_tables(&self) -> Result<()> {
        // Create heart_rate table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS heart_rate (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bpm INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create heart_rate table")?;

        // Create index
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_heart_rate_created_at 
            ON heart_rate (created_at)
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create index on heart_rate table")?;

        tracing::info!("Database tables initialized");
        Ok(())
    }

    /// Insert a new heart rate record
    pub async fn insert_heart_rate(&self, bpm: i32) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO heart_rate (bpm) VALUES (?)"
        )
        .bind(bpm)
        .execute(&self.pool)
        .await
        .context("Failed to insert heart rate record")?;

        let id = result.last_insert_rowid();
        tracing::debug!("Inserted heart rate record: bpm={}, id={}", bpm, id);
        Ok(id)
    }

    /// Get recent heart rate records
    pub async fn get_recent_heart_rates(&self, limit: i32) -> Result<Vec<HeartRateRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, bpm, created_at 
            FROM heart_rate 
            ORDER BY created_at DESC 
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch recent heart rate records")?;

        let records = rows.into_iter().map(|row| {
            HeartRateRecord {
                id: row.get("id"),
                bpm: row.get("bpm"),
                created_at: row.get("created_at"),
            }
        }).collect();

        Ok(records)
    }

    /// Get heart rate statistics
    pub async fn get_stats(&self) -> Result<HeartRateStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_records,
                AVG(bpm) as avg_bpm,
                MIN(bpm) as min_bpm,
                MAX(bpm) as max_bpm
            FROM heart_rate
            WHERE created_at >= datetime('now', '-24 hours')
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch heart rate statistics")?;

        Ok(HeartRateStats {
            total_records: row.get("total_records"),
            avg_bpm: row.get::<Option<f64>, _>("avg_bpm").unwrap_or(0.0),
            min_bpm: row.get::<Option<i32>, _>("min_bpm").unwrap_or(0),
            max_bpm: row.get::<Option<i32>, _>("max_bpm").unwrap_or(0),
        })
    }

    /// Close database connection
    pub async fn close(self) {
        self.pool.close().await;
        tracing::info!("Database connection closed");
    }
}

#[derive(Debug)]
pub struct HeartRateStats {
    pub total_records: i32,
    pub avg_bpm: f64,
    pub min_bpm: i32,
    pub max_bpm: i32,
}
