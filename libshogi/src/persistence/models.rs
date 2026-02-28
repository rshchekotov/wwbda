use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::persistence::schema::shogi_game_move)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ShogiGameMove {
    pub id: String,
    pub turn: i32,
    pub ts: NaiveDateTime,
    pub sfen: String,
}

#[derive(Queryable, Selectable, Insertable, Default)]
#[diesel(table_name = crate::persistence::schema::shogi_game)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ShogiGame {
    pub id: String,
    pub sente: Option<String>,
    pub gote: Option<String>,
    pub winner: Option<String>,
    pub win_condition: Option<String>,
}
