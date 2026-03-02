PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS player (
  id BIGINT PRIMARY KEY,
  lishogi_tag TEXT NOT NULL UNIQUE
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS shogi_game (
  id TEXT PRIMARY KEY,
  sente TEXT,
  gote TEXT,
  -- winner: 0 = in-progress/draw[win_condition], 1 = sente, 2 = gote
  winner INTEGER NOT NULL DEFAULT 0,
  win_condition TEXT,
  CHECK (sente IS NULL OR sente != gote),
  CHECK (winner == 0 OR win_condition IS NOT NULL),
  FOREIGN KEY(sente)
    REFERENCES player(lishogi_tag)
      ON DELETE NO ACTION
      ON UPDATE CASCADE,
  FOREIGN KEY(gote)
    REFERENCES player(lishogi_tag)
      ON DELETE NO ACTION
      ON UPDATE CASCADE
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_shogi_sente_gote ON shogi_game(sente, gote);

CREATE TABLE IF NOT EXISTS shogi_game_move (
  id TEXT PRIMARY KEY,
  turn INTEGER NOT NULL,
  ts DATETIME NOT NULL UNIQUE DEFAULT CURRENT_TIMESTAMP,
  sfen TEXT NOT NULL,
  FOREIGN KEY(id)
    REFERENCES shogi_game(id)
      ON DELETE NO ACTION
      ON UPDATE CASCADE
) WITHOUT ROWID;
