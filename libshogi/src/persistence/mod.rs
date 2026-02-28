use diesel::{
    RunQueryDsl, SelectableHelper, SqliteConnection,
    r2d2::{ConnectionManager, PoolError},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use r2d2::{Error as PooledConnectionError, Pool, PooledConnection};
use std::env;
use std::error::Error as StdError;

use crate::{State, persistence::models::ShogiGame};

pub mod models;
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn establish_connection() -> SqlitePool {
    let database_url = env::var("DATABASE_URL").expect("Expected SQLite URL in the environment.");
    init_pool(&database_url).expect("Expected to be able to create database pool.")
}

fn init_pool(database_url: &str) -> Result<SqlitePool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn sqlite_pool_handler(
    pool: &SqlitePool,
) -> Result<SqlitePooledConnection, PooledConnectionError> {
    pool.get()
}

pub fn run_migrations(
    mut conn: SqlitePooledConnection,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

pub fn add_game(game: String, state: &mut State) {
    use crate::persistence::schema::shogi_game;

    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled Connection should be established.");

    let new_game = ShogiGame {
        id: game.clone(),
        ..Default::default()
    };

    diesel::insert_into(shogi_game::table)
        .values(&new_game)
        .returning(ShogiGame::as_returning())
        .get_result(connection)
        .expect("Error registering game in database.");

    state.threads.push(tokio::spawn({
        let callback = state.message_callback;
        async move {
            let _ = crate::ws::listen_to_game(game.as_str(), callback).await;
        }
    }));
}
