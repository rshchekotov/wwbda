# 吾輩はボットである

This is a Discord Bot, based on Poise, which itself is built on Serenity.
The Discord Bot will contain multiple features, which - when sufficiently 'self-sufficient'
should be kept in separate modules ('crates').
When possible, such features should be unit tested,
as soon as Discord comes into play - Unit Testing becomes essentially impossible,
hence those features should be tested to work as reliable as possible
in an insulated environment.

## Features

### Shogi

**Tracking asynchronous Shogi games on Discord (backed by LiShogi) via WebSockets**.

The bot, thus:

- listens to LiShogi watch WebSockets for move updates,
- stores moves and game state in the database,
- posts Discord updates and optionally pings players based on their preferences,
- generates a Shogi board image from SFEN for each move and sends it as an attachment.

The (Li)Shogi-specific parts are implemented in the `libshogi` crate,
while the Discord interactions are implemented in the `bot` crate.
To further improve decoupling, functions that interact with the database should
be inside the `libshogi` crate.
If necessary (i.e. when creating a game with players) -
those players are passed as arguments in a function call from the Discord module,
but the whole DB logic will happen inside `libshogi`.

The bot must support multiple active games in parallel.
Each game corresponds to an independent LiShogi WebSocket listener.
Implementation-wise, the runtime maintains a game-task registry such that:
each active game has exactly one listener task,
reconnects do not spawn duplicates,
move inserts are idempotent (unique (game_id, ply)),
listener failures are isolated to that game and do not affect other games.
A centralized "socket manager" is responsible for starting/stopping listeners,
supervising reconnects, and routing parsed move events into the persistence +
notification pipeline.

At runtime, the bot:

- posts updates on opponent moves in a self-created thread
  (based on the game ID) inside a channel `#shogi`
  (optionally pinging based on user preferences),
- optionally emits periodic reminders when it is a user's turn
  and they haven't moved for a configured duration,
- optionally emits pre-timeout warnings when it is a user's turn
  and their remaining time approaches a configured threshold.

> [!NOTE]
> Ping Rate-Limiting is not yet implemented.

The ping logic must be rate-limited and stateful (persisted in RAM/cache)
to avoid repeated notifications during periodic checks.

> [!NOTE]
> The image rendering is not yet implemented.

For move updates, the bot renders the Shogi position from SFEN into an image
and attaches it to Discord messages.
This is intended to be lightweight and deterministic:
parse SFEN into a 9×9 board + hands + side-to-move,
render a board with pieces (textures in svg format will later be provided
[from LiShogi](https://github.com/WandererXII/lishogi/blob/master/ui/%40build/pieces/assets/standard/ryoko_1kanji)),
optionally overlay last move info (ply/USI) and whose turn it is.

Rendering should be implemented in Rust (e.g., via an image library)
and kept inside the libshogi module so it can be unit-tested from SFEN fixtures.

## Commands

- `/shogi user lishogi_tag user?`
  - Argument Spec:
    - `lishogi_tag:str` (LiShogi Username)
    - `user:User` (Whom (among Discord users) to assign the LiShogi Tag to)
- `/shogi track id sente? gote?`
  - Argument Spec:
    - `id:str`
    - `sente:<lishogi-user-ref>`
    - `gote:<lishogi-user-ref>`
  - Sente/Gote are optional and refer to LiShogi users
- `/shogi ping on-move do-not-forget pre-timeout` (ephemeral, user-specific)
  - on-move is a boolean option, and allows to be pinged on the move of the
    opponent (default true)
  - do-not-forget pings in the specified interval during ones own turn
    (default null)
  - pre-timeout pings once the clock is about to run out, calculated by taking
    the last made move and its timestamp and alerting on the indicated time
  - error handling should be implemented to make sure the time provided is
    valid and does not exceed the clock time
    (and for the do-not-forget one, should be more than 5m).

> [!NOTE]
> User Pings are not customizable yet, i.e. the second command does not yet exist.

## Development

### Release Checklist

- [ ] Update `Cargo.toml` in the respective projects
- [ ] Run Cargo Checks and Formatting
- [ ] Create Git Tag
- [ ] Check Documentation / CHANGELOG for user-facing changes
