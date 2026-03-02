use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Identifiable, Insertable, Serialize, Associations, Clone)]
#[diesel(table_name = crate::persistence::schema::shogi_game_move)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(ShogiGame, foreign_key = id))]
pub struct ShogiGameMove {
    pub id: String,
    pub turn: i32,
    pub ts: NaiveDateTime,
    pub sfen: String,
}

#[derive(Queryable, Selectable, Identifiable, Insertable, Default, Serialize, Clone)]
#[diesel(table_name = crate::persistence::schema::shogi_game)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ShogiGame {
    pub id: String,
    pub sente: Option<String>,
    pub gote: Option<String>,
    pub winner: i32,
    pub win_condition: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Clone)]
#[diesel(table_name = crate::persistence::schema::player)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Player {
    pub id: i64,
    pub lishogi_tag: String,
}

#[derive(Serialize, Clone)]
pub struct DetailedShogiGame {
    pub game: ShogiGame,
    pub latest_move: Option<ShogiGameMove>,
    pub sente: Option<Player>,
    pub gote: Option<Player>,
}