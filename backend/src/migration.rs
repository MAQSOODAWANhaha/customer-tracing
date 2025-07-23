use sea_orm::{Database, DatabaseConnection, DbErr, ConnectionTrait};
use std::fs;
use std::path::Path;
use tracing::{info, warn, error};
use std::env;

pub struct DatabaseMigrator {
    db_url: String,
}

impl DatabaseMigrator {
    pub fn new(db_url: String) -> Self {
        Self { db_url }
    }

    /// 确保数据库文件和目录存在
    pub async fn ensure_database_exists(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 解析SQLite数据库路径
        if self.db_url.starts_with("sqlite://") {
            let db_path = self.db_url.strip_prefix("sqlite://").unwrap();
            let db_file = Path::new(db_path);
            
            // 创建父目录（如果不存在）
            if let Some(parent_dir) = db_file.parent() {
                if !parent_dir.exists() {
                    info!("创建数据库目录: {}", parent_dir.display());
                    fs::create_dir_all(parent_dir)?;
                }
            }
            
            // 如果数据库文件不存在，创建空文件
            if !db_file.exists() {
                info!("创建数据库文件: {}", db_file.display());
                fs::File::create(db_file)?;
            }
        }
        
        Ok(())
    }

    /// 运行所有迁移
    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 确保数据库存在
        self.ensure_database_exists().await?;
        
        // 连接数据库
        let db = Database::connect(&self.db_url).await?;
        
        // 创建迁移表（如果不存在）
        self.create_migration_table(&db).await?;
        
        // 获取迁移文件列表
        let migrations = self.get_migration_files()?;
        
        for migration in migrations {
            if !self.is_migration_applied(&db, &migration.name).await? {
                info!("应用迁移: {}", migration.name);
                self.apply_migration(&db, &migration).await?;
                self.record_migration(&db, &migration.name).await?;
            } else {
                info!("跳过已应用的迁移: {}", migration.name);
            }
        }
        
        info!("所有数据库迁移已完成");
        Ok(())
    }

    /// 创建迁移记录表
    async fn create_migration_table(&self, db: &DatabaseConnection) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(255) PRIMARY KEY,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        "#;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    /// 检查迁移是否已应用
    async fn is_migration_applied(&self, db: &DatabaseConnection, migration_name: &str) -> Result<bool, DbErr> {
        let sql = "SELECT COUNT(*) as count FROM schema_migrations WHERE version = ?";
        
        use sea_orm::FromQueryResult;
        
        #[derive(FromQueryResult)]
        struct CountResult {
            count: i32,
        }
        
        let result: Option<CountResult> = CountResult::find_by_statement(
            sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                sql,
                vec![migration_name.into()]
            )
        )
        .one(db)
        .await?;
        
        Ok(result.map_or(false, |r| r.count > 0))
    }

    /// 记录已应用的迁移
    async fn record_migration(&self, db: &DatabaseConnection, migration_name: &str) -> Result<(), DbErr> {
        let sql = "INSERT INTO schema_migrations (version) VALUES (?)";
        
        db.execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![migration_name.into()]
        )).await?;
        
        Ok(())
    }

    /// 应用单个迁移
    async fn apply_migration(&self, db: &DatabaseConnection, migration: &Migration) -> Result<(), Box<dyn std::error::Error>> {
        // 读取SQL文件内容
        let sql_content = fs::read_to_string(&migration.path)?;
        
        // 更智能的SQL语句分割：处理注释和空行
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        
        for line in sql_content.lines() {
            let line = line.trim();
            
            // 跳过空行和注释行
            if line.is_empty() || line.starts_with("--") {
                continue;
            }
            
            current_statement.push_str(line);
            current_statement.push(' ');
            
            // 如果行以分号结尾，表示语句结束
            if line.ends_with(';') {
                let stmt = current_statement.trim().trim_end_matches(';').trim();
                if !stmt.is_empty() {
                    statements.push(stmt.to_string());
                }
                current_statement.clear();
            }
        }
        
        // 如果还有未处理的语句（没有以分号结尾）
        let remaining = current_statement.trim();
        if !remaining.is_empty() {
            statements.push(remaining.to_string());
        }
        
        // 按顺序执行每个语句
        for (i, statement) in statements.iter().enumerate() {
            info!("执行SQL语句 {}: {}", i + 1, &statement[..statement.len().min(100)]);
            match db.execute_unprepared(statement).await {
                Ok(_) => info!("SQL语句 {} 执行成功", i + 1),
                Err(e) => {
                    error!("SQL语句 {} 执行失败: {}", i + 1, e);
                    return Err(e.into());
                }
            }
        }
        
        Ok(())
    }

    /// 获取所有迁移文件
    fn get_migration_files(&self) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
        let migrations_dir = Path::new("migrations");
        
        if !migrations_dir.exists() {
            warn!("迁移目录不存在: migrations/");
            return Ok(Vec::new());
        }
        
        let mut migrations = Vec::new();
        
        for entry in fs::read_dir(migrations_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "sql") {
                if let Some(file_name) = path.file_stem().and_then(|n| n.to_str()) {
                    migrations.push(Migration {
                        name: file_name.to_string(),
                        path: path.clone(),
                    });
                }
            }
        }
        
        // 按文件名排序以确保正确的执行顺序
        migrations.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(migrations)
    }

    /// 检查数据库状态
    pub async fn check_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 确保数据库存在
        self.ensure_database_exists().await?;
        
        let db = Database::connect(&self.db_url).await?;
        
        // 检查迁移表是否存在
        let migration_table_exists = self.migration_table_exists(&db).await?;
        
        if migration_table_exists {
            // 获取已应用的迁移
            let applied_migrations = self.get_applied_migrations(&db).await?;
            
            info!("数据库连接: 正常");
            info!("已应用的迁移数量: {}", applied_migrations.len());
            
            for migration in applied_migrations {
                info!("  - {}", migration);
            }
        } else {
            info!("数据库连接: 正常");
            info!("迁移表尚未创建，请运行迁移");
        }
        
        Ok(())
    }

    /// 检查迁移表是否存在
    async fn migration_table_exists(&self, db: &DatabaseConnection) -> Result<bool, DbErr> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_migrations'";
        
        use sea_orm::FromQueryResult;
        
        #[derive(FromQueryResult)]
        struct TableResult {
            #[allow(dead_code)]
            name: String,
        }
        
        let result: Option<TableResult> = TableResult::find_by_statement(
            sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                sql,
                vec![]
            )
        )
        .one(db)
        .await?;
        
        Ok(result.is_some())
    }

    /// 获取已应用的迁移列表
    async fn get_applied_migrations(&self, db: &DatabaseConnection) -> Result<Vec<String>, DbErr> {
        let sql = "SELECT version FROM schema_migrations ORDER BY version";
        
        use sea_orm::FromQueryResult;
        
        #[derive(FromQueryResult)]
        struct MigrationRecord {
            version: String,
        }
        
        let results: Vec<MigrationRecord> = MigrationRecord::find_by_statement(
            sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                sql,
                vec![]
            )
        )
        .all(db)
        .await?;
        
        Ok(results.into_iter().map(|r| r.version).collect())
    }
}

#[derive(Debug)]
struct Migration {
    name: String,
    path: std::path::PathBuf,
}

/// 便捷函数：运行数据库迁移
pub async fn run_database_migrations() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./data/customer_tracker.db".to_string());
    
    let migrator = DatabaseMigrator::new(database_url);
    migrator.run_migrations().await
}

/// 便捷函数：检查数据库状态
pub async fn check_database_status() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./data/customer_tracker.db".to_string());
    
    let migrator = DatabaseMigrator::new(database_url);
    migrator.check_status().await
}