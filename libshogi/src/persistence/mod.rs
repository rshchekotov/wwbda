use diesel::{
    BelongingToDsl, SqliteConnection,
    associations::HasTable,
    prelude::*,
    r2d2::{ConnectionManager, PoolError},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::debug;
use r2d2::{Error as PooledConnectionError, Pool, PooledConnection};
use std::error::Error as StdError;
use std::{env, sync::Arc};
use tokio::task::JoinHandle;

use crate::{
    EndGameData, MoveData, SocketMessageCallback,
    persistence::models::{DetailedShogiGame, Player, ShogiGame, ShogiGameMove},
};

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

pub fn run_migrations() -> Result<(), Box<dyn StdError + Send + Sync>> {
    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled Connection should be established.");
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

pub fn add_game(
    game: String,
    sente_lishogi: Option<String>,
    gote_lishogi: Option<String>,
    threads: &mut Vec<JoinHandle<()>>,
    message_callback: Option<Arc<SocketMessageCallback>>,
) -> bool {
    use crate::persistence::schema::shogi_game;

    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled Connection should be established.");

    let new_game = ShogiGame {
        id: game.clone(),
        sente: sente_lishogi,
        gote: gote_lishogi,
        ..Default::default()
    };

    let result = diesel::insert_into(shogi_game::table)
        .values(&new_game)
        .returning(ShogiGame::as_returning())
        .get_result(connection);

    let added = result.is_ok();
    if added {
        threads.push(tokio::spawn({
            let callback = message_callback;
            async move {
                let _ = crate::ws::listen_to_game(game.as_str(), callback).await;
            }
        }));
    } else {
        debug!("DB Error during Add Game: {:?}", result.err());
    }
    added
}

pub fn add_player(discord: i64, lishogi: String) -> bool {
    use crate::persistence::schema::player;

    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled connection should be established.");

    let new_player = Player {
        id: discord,
        lishogi_tag: lishogi,
    };

    let result = diesel::insert_into(player::table)
        .values(new_player)
        .returning(Player::as_returning())
        .get_result(connection);

    let added = result.is_ok();

    if !added {
        debug!("DB Error during Add Player: {:?}", result.err());
    }

    added
}

pub fn get_game(game_id: &str) -> Option<ShogiGame> {
    use crate::persistence::schema::shogi_game::dsl::shogi_game;

    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled connection should be established.");

    shogi_game
        .find(game_id)
        .select(ShogiGame::as_select())
        .first(connection)
        .optional()
        .expect("Query by Game ID should not fail")
}

pub async fn get_game_details(game_id: &str) -> Option<DetailedShogiGame> {
    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled connection should be established.");

    use crate::persistence::schema::player::dsl::*;
    use crate::persistence::schema::shogi_game::dsl::shogi_game;
    use crate::persistence::schema::shogi_game_move::dsl::ts;

    let result = shogi_game
        .select(ShogiGame::as_select())
        .find(game_id)
        .first(connection)
        .ok();

    if let Some(game) = result {
        let moves = ShogiGameMove::belonging_to(&game)
            .select(ShogiGameMove::as_select())
            .order_by(ts.desc())
            .load(connection)
            .expect("Should be able to query moves for a game.");

        let sente = if let Some(sente_tag) = game.sente.clone() {
            player::table()
                .select(Player::as_select())
                .filter(lishogi_tag.eq(sente_tag))
                .first(connection)
                .optional()
                .expect("Query for Sente Player should not fail.")
        } else {
            None
        };

        let gote = if let Some(gote_tag) = game.gote.clone() {
            player::table()
                .select(Player::as_select())
                .filter(lishogi_tag.eq(gote_tag))
                .first(connection)
                .optional()
                .expect("Query for Gote Player should not fail.")
        } else {
            None
        };

        Some(DetailedShogiGame {
            game,
            latest_move: moves.first().cloned(),
            sente,
            gote,
        })
    } else {
        None
    }
}

pub async fn add_move(game_id: &str, data: MoveData) -> bool {
    use crate::persistence::schema::shogi_game::dsl::*;
    use crate::persistence::schema::shogi_game_move;

    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled Connection should be established.");

    let game: ShogiGame = shogi_game
        .find(game_id)
        .first(connection)
        .expect("Game must exist for moves to be added.");

    let game_move = ShogiGameMove {
        id: game.id,
        sfen: data.sfen,
        turn: data.ply as i32,
        ts: chrono::offset::Utc::now().naive_utc(),
    };

    diesel::insert_into(shogi_game_move::table)
        .values(game_move)
        .execute(connection)
        .is_ok()
}

pub async fn end_game(game_id: &str, data: EndGameData) {
    use crate::persistence::schema::shogi_game::dsl::*;
    let pool = establish_connection();
    let connection =
        &mut sqlite_pool_handler(&pool).expect("Pooled Connection should be established.");

    let winner_id = if data.winner == "sente" { 1 } else { 2 };

    diesel::update(shogi_game)
        .filter(id.eq(game_id))
        .set((winner.eq(winner_id), win_condition.eq(data.status.name)))
        .execute(connection)
        .expect("Should be able to write EndGameData.");
}
