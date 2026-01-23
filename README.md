# 吾輩はボットである

This is a Discord Bot, based on Poise, which itself is built on Serenity.
The Discord Bot will contain multiple features, which - when sufficiently 'self-sufficient'
should be kept in separate modules ('crates').
When possible, such features should be unit tested,
as soon as Discord comes into play - Unit Testing becomes essentially impossible,
hence those features should be tested to work as reliable as possible in an insulated environment.

For the Discord Bot, you may use
[this repo](https://github.com/QueenOfDoom/kanshi) as a reference of a proper implementation.
I'd like commands, ORM and those types of features implemented in the same fashion.

The Deployment will ultimately happen in a Docker Container, so a Dockerfile should be created eventually.

## Features

### Shogi

**Tracking asynchronous Shogi games on Discord (backed by LiShogi) via WebSockets**.

The bot, thus:
- listens to LiShogi watch WebSockets for move updates,
- stores moves and game state in the database (schema defined elsewhere in this repo),
- posts Discord updates and optionally pings players based on their preferences,
- generates a Shogi board image from SFEN for each move and sends it as an attachment.

A minimal Python implementation (./libshogi/src/ref.py) exists as a behavioral reference
(especially for the WebSocket quirks - they should probably be implemented via `tokio-tungstenite`).
The primary implementation target is Rust.

The (Li)Shogi-specific parts should be implemented in the `libshogi` crate,
while the Discord interaction should be implemented in the `bot` crate.
To further decrease decoupling, functions that interact with the database should be inside the `libshogi`
crate - if necessary (i.e. when creating a game with players) - those players will come from a
function call from the Discord module, but the whole DB logic will happen inside `libshogi`.

The bot must support multiple active games in parallel.
Each game corresponds to an independent LiShogi WebSocket listener.
Implementation-wise, the runtime maintains a game-task registry such that:
each active game has exactly one listener task,
reconnects do not spawn duplicates,
move inserts are idempotent (unique (game_id, ply)),
listener failures are isolated to that game and do not affect other games.
A centralized “socket manager” is responsible for starting/stopping listeners,
supervising reconnects, and routing parsed move events into the persistence + notification pipeline.

At runtime, the bot:
- posts updates on opponent moves in a self-created thread (based on the GameID) inside a channel `#shogi` (optionally pinging based on user preferences),
- optionally emits periodic reminders when it is a user’s turn and they haven’t moved for a configured duration,
- optionally emits pre-timeout warnings when it is a user’s turn and their remaining time approaches a configured threshold.
The ping logic must be rate-limited and stateful (persisted in RAM/cache) to avoid repeated notifications during periodic checks.

For move updates, the bot renders the Shogi position from SFEN into an image and attaches it to Discord messages.
This is intended to be lightweight and deterministic:
parse SFEN into a 9×9 board + hands + side-to-move,
render a board with pieces (textures in svg format will later be provided
[from LiShogi](https://github.com/WandererXII/lishogi/blob/master/ui/%40build/pieces/assets/standard/ryoko_1kanji)),
optionally overlay last move info (ply/USI) and whose turn it is.

Rendering should be implemented in Rust (e.g., via an image library) and kept inside the libshogi module so it can be unit-tested from SFEN fixtures.

## Commands

- `/shogi create sente:<user-1|optional> gote:<user-2|optional> game-id:<str>`
    - User 1, User 2 are optional and if indicated - are pingable Discord Users
- `/shogi ping on-move:<bool=true|optional> do-not-forget:<str[i.e. 1d or 3h or 1w]|optional> pre-timeout:<str[i.e. 30m]>` (ephemeral, user-specific)
    - on-move is a boolean option, and allows to be pinged on the move of the opponent (default true)
    - do-not-forget pings in the specified interval during ones own turn (default null)
    - pre-timeout pings once the clock is about to run out, calculated by taking the last made move and its timestamp and alerting on the indicated time
    - error handling should be implemented to make sure the time provided is valid and does not exceed the clock time (and for the do-not-forget one, should be more than 5m).

## Database Schema (SQLite via Diesel)
Move ingestion must be robust against reconnection replay and duplicate delivery:
inserting a move that already exists should be a no-op.

### Shogi Database (shogi.db)
```sql
-- Users are Discord accounts
CREATE TABLE users (
  id            BIGINT PRIMARY KEY,        -- discord user id
  display_name  TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Ping preferences for your /shogi-ping command
CREATE TABLE user_prefs (
  user_id              BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

  -- defaults
  ping_on_move         BOOLEAN NOT NULL DEFAULT TRUE,

  -- optional cadence ping if they haven't moved yet
  ping_do_not_forget   INTERVAL NULL,       -- e.g. '1 day'

  -- optional "warn before flag" for their own clock
  ping_pre_timeout     INTERVAL NULL,       -- e.g. '1 hour'

  updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Game table: two players only
CREATE TYPE game_status AS ENUM ('ongoing', 'finished', 'aborted');

CREATE TABLE games (
  id                   TEXT PRIMARY KEY DEFAULT, -- lishogi ID (or if manual game, UUID)

  -- where this game lives
  guild_id             BIGINT NOT NULL,
  channel_id           BIGINT NOT NULL,
  thread_id            BIGINT NULL,         -- if you use threads
  message_id           BIGINT NULL,         -- "game message" to update

  -- players
  player_sente_id      BIGINT NOT NULL REFERENCES users(id),
  player_gote_id       BIGINT NOT NULL REFERENCES users(id),
  CHECK (player_sente_id <> player_gote_id),

  -- state
  status               game_status NOT NULL DEFAULT 'ongoing',
  winner_side          TEXT NULL CHECK (winner_side IN ('sente','gote')),
  end_reason           TEXT NULL,           -- e.g. 'resign', 'timeout', 'checkmate', 'draw'
  finished_at          TIMESTAMPTZ NULL,

  -- starting position (optional but useful)
  initial_sfen         TEXT NOT NULL DEFAULT 'startpos',

  -- time control (optional; depends on your bot)
  -- store in seconds to keep it simple + portable
  clock_sente_initial  INTEGER NULL,
  clock_gote_initial   INTEGER NULL,
  clock_increment      INTEGER NULL,        -- byoyomi/inc seconds etc., if you use it

  created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX games_by_channel_status ON games (guild_id, channel_id, status);
CREATE INDEX games_by_players_status ON games (player_sente_id, player_gote_id, status);
CREATE INDEX games_last_move_at ON games (status, last_move_at);

-- Moves: store the minimal replay data + convenient metadata
CREATE TABLE moves (
  id                BIGSERIAL PRIMARY KEY,
  game_id           UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,

  ply               INTEGER NOT NULL,             -- 1..n (or 0..n, your choice)
  side              TEXT NOT NULL CHECK (side IN ('sente','gote')), -- although technically computable through ply, probably convenient to have

  usi               TEXT NOT NULL,                -- e.g. "5i4h"
  sfen              TEXT NULL,                    -- optional: position after move
  is_check          BOOLEAN NULL,                 -- if the API includes it

  moved_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  source            TEXT NOT NULL DEFAULT 'lishogi', -- or 'manual', etc.

  -- sanity: each ply only once per game
  UNIQUE (game_id, ply)
);

CREATE INDEX moves_by_game_ply ON moves (game_id, ply);
```