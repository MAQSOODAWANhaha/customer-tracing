use customer_tracker::{
    cli::{handle_cli_command, Cli},
    config::Config,
    database::create_database_connection,
    handlers::auth::AppState,
    routes::create_routes,
};
use clap::Parser;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Check if it's a CLI command or server start
    match &cli.command {
        customer_tracker::cli::Commands::Server(server_args) => {
            match &server_args.action {
                customer_tracker::cli::ServerAction::Start { port, host } => {
                    start_server(*port, host.clone()).await?;
                }
                _ => {
                    handle_cli_command(cli).await?;
                }
            }
        }
        _ => {
            handle_cli_command(cli).await?;
        }
    }

    Ok(())
}

async fn start_server(port: u16, host: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting customer tracker server...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Create database connection
    let db = create_database_connection(&config.database_url).await?;
    info!("Database connected successfully");

    // Create application state
    let app_state = AppState {
        db,
        jwt_secret: config.jwt_secret,
        jwt_expire_hours: config.jwt_expire_hours,
    };

    // Create routes
    let app = create_routes(app_state);

    // Create socket address
    let addr = SocketAddr::new(
        host.parse().unwrap_or_else(|_| "0.0.0.0".parse().unwrap()),
        port,
    );

    info!("Server starting on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
