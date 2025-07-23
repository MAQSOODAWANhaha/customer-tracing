use clap::{Args, Parser, Subcommand};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    config::Config,
    database::create_database_connection,
    entities::{user, user::Entity as User},
    migration::{run_database_migrations, check_database_status},
    utils::password::hash_password,
};

#[derive(Parser)]
#[command(name = "customer-tracker")]
#[command(about = "客户追踪系统管理工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 用户管理
    User(UserArgs),
    /// 数据库管理
    Database(DatabaseArgs),
    /// 服务器管理
    Server(ServerArgs),
}

#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub action: UserAction,
}

#[derive(Subcommand)]
pub enum UserAction {
    /// 创建新用户
    Create {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        name: String,
    },
    /// 列出所有用户
    List {
        #[arg(short, long, default_value = "10")]
        limit: u64,
    },
    /// 重置用户密码
    ResetPassword {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
    /// 禁用/启用用户
    Toggle {
        #[arg(short, long)]
        username: String,
    },
}

#[derive(Args)]
pub struct DatabaseArgs {
    #[command(subcommand)]
    pub action: DatabaseAction,
}

#[derive(Subcommand)]
pub enum DatabaseAction {
    /// 运行数据库迁移
    Migrate,
    /// 数据库状态
    Status,
}

#[derive(Args)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub action: ServerAction,
}

#[derive(Subcommand)]
pub enum ServerAction {
    /// 启动服务器
    Start {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },
    /// 生成JWT密钥
    GenerateJwtSecret,
}

pub async fn handle_cli_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::User(user_args) => handle_user_command(user_args).await,
        Commands::Database(db_args) => handle_database_command(db_args).await,
        Commands::Server(server_args) => handle_server_command(server_args).await,
    }
}

async fn handle_user_command(args: UserArgs) -> Result<(), Box<dyn std::error::Error>> {
    // 如果数据库不存在，先运行迁移
    let config = Config::from_env()?;
    let db_url = &config.database_url;
    
    // 确保数据库和表存在
    use crate::migration::DatabaseMigrator;
    let migrator = DatabaseMigrator::new(db_url.clone());
    migrator.ensure_database_exists().await?;
    migrator.run_migrations().await?;
    
    let db = create_database_connection(db_url).await?;

    match args.action {
        UserAction::Create { username, password, name } => {
            create_user(&db, &username, &password, &name).await?;
        }
        UserAction::List { limit } => {
            list_users(&db, limit).await?;
        }
        UserAction::ResetPassword { username, password } => {
            reset_user_password(&db, &username, &password).await?;
        }
        UserAction::Toggle { username } => {
            toggle_user_status(&db, &username).await?;
        }
    }

    Ok(())
}

async fn create_user(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if user already exists
    let existing_user = User::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await?;

    if existing_user.is_some() {
        println!("错误: 用户名 '{}' 已存在", username);
        return Ok(());
    }

    let password_hash = hash_password(password)?;
    let now = Utc::now();

    let new_user = user::ActiveModel {
        username: Set(username.to_string()),
        password_hash: Set(password_hash),
        name: Set(name.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        is_active: Set(true),
        last_login_at: Set(None),
        ..Default::default()
    };

    let user = new_user.insert(db).await?;
    println!("用户创建成功: {} ({})", user.name, user.username);

    Ok(())
}

async fn list_users(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let users = User::find()
        .limit(Some(limit))
        .all(db)
        .await?;

    if users.is_empty() {
        println!("没有找到用户");
        return Ok(());
    }

    println!("用户列表:");
    println!("{:<5} {:<20} {:<30} {:<10} {:<20}", "ID", "用户名", "姓名", "状态", "最后登录");
    println!("{:-<85}", "");

    for user in users {
        let status = if user.is_active { "启用" } else { "禁用" };
        let last_login = user.last_login_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "从未".to_string());

        println!(
            "{:<5} {:<20} {:<30} {:<10} {:<20}",
            user.id, user.username, user.name, status, last_login
        );
    }

    Ok(())
}

async fn reset_user_password(
    db: &DatabaseConnection,
    username: &str,
    new_password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = User::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await?
        .ok_or("用户不存在")?;

    let password_hash = hash_password(new_password)?;
    let mut user_active: user::ActiveModel = user.into();
    
    user_active.password_hash = Set(password_hash);
    user_active.updated_at = Set(Utc::now());

    user_active.update(db).await?;
    println!("用户 '{}' 的密码已重置", username);

    Ok(())
}

async fn toggle_user_status(
    db: &DatabaseConnection,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = User::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await?
        .ok_or("用户不存在")?;

    let mut user_active: user::ActiveModel = user.clone().into();
    user_active.is_active = Set(!user.is_active);
    user_active.updated_at = Set(Utc::now());

    user_active.update(db).await?;
    
    let status = if !user.is_active { "启用" } else { "禁用" };
    println!("用户 '{}' 已{}", username, status);

    Ok(())
}

async fn handle_database_command(args: DatabaseArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.action {
        DatabaseAction::Migrate => {
            println!("开始执行数据库迁移...");
            run_database_migrations().await?;
            println!("数据库迁移完成");
        }
        DatabaseAction::Status => {
            check_database_status().await?;
            
            // 额外显示用户统计（仅在表存在时）
            let config = Config::from_env()?;
            match create_database_connection(&config.database_url).await {
                Ok(db) => {
                    match User::find().count(&db).await {
                        Ok(count) => println!("用户总数: {}", count),
                        Err(_) => println!("用户表尚未创建"),
                    }
                }
                Err(e) => println!("数据库连接失败: {}", e),
            }
        }
    }

    Ok(())
}

async fn handle_server_command(args: ServerArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.action {
        ServerAction::Start { port, host } => {
            println!("服务器启动功能需要在main.rs中实现");
            println!("计划在 {}:{} 启动服务器", host, port);
        }
        ServerAction::GenerateJwtSecret => {
            let secret = Uuid::new_v4().to_string();
            println!("生成的JWT密钥: {}", secret);
            println!("请将此密钥设置为环境变量 JWT_SECRET");
        }
    }

    Ok(())
}