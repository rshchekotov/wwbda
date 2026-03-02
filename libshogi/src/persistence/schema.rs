// @generated automatically by Diesel CLI.

diesel::table! {
    player (id) {
        id -> BigInt,
        lishogi_tag -> Text,
    }
}

diesel::table! {
    shogi_game (id) {
        id -> Text,
        sente -> Nullable<Text>,
        gote -> Nullable<Text>,
        winner -> Integer,
        win_condition -> Nullable<Text>,
    }
}

diesel::table! {
    shogi_game_move (id, turn) {
        id -> Text,
        turn -> Integer,
        ts -> Timestamp,
        sfen -> Text,
    }
}

diesel::joinable!(shogi_game_move -> shogi_game (id));

diesel::allow_tables_to_appear_in_same_query!(player, shogi_game, shogi_game_move,);
