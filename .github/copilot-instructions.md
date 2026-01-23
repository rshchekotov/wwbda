# Copilot / AI agent instructions for WWBDA

Purpose
- Help contributors and AI agents quickly understand and extend this Discord Shogi bot project.

Big picture
- Two crates: `libshogi` (Shogi logic, rendering, DB) and `bot` (Discord integration via Poise/Serenity).
- `libshogi` is the canonical place for game logic, persistence, and image rendering from SFEN. See [libshogi/src/lib.rs](libshogi/src/lib.rs).
- `bot` is responsible for Discord commands, user-facing threads/messages, and calling into `libshogi`. See [bot/src/main.rs](bot/src/main.rs).

Key runtime responsibilities and patterns
- One WebSocket listener per active game (lishogi watch socket). A centralized "socket manager" should start/stop listeners and avoid duplicate listeners.
- Move ingestion must be idempotent: database has UNIQUE (game_id, ply). Ensure inserts treat conflicts as no-ops.
- Persist user ping preferences in `user_prefs` (schema in [README.md](README.md)). Ping logic must be rate-limited and stateful.
- Rendering: parse SFEN → 9×9 board + hands → produce deterministic image. Keep rendering inside `libshogi` for unit testing.

Files to inspect for examples and behavior
- High-level behavior and DB schema: [README.md](README.md)
- WebSocket Python reference (behavior + keepalive): [libshogi/src/ref.py](libshogi/src/ref.py)
- Rust library entry: [libshogi/src/lib.rs](libshogi/src/lib.rs)
- Bot entry: [bot/src/main.rs](bot/src/main.rs)
 - Poise usage and reference implementation: see Poise docs and the Kanshi example repo at https://github.com/QueenOfDoom/kanshi

Build, run, test workflows
- Build entire workspace: `cargo build --workspace`
- Run bot crate locally: `cargo run -p bot` (may require Discord token env vars and DB)
- Run tests: `cargo test --workspace`
- Add dependencies with `cargo add <crate> -p <crate-name>` in workspace context (example: `tokio-tungstenite` used for websockets)

Project-specific conventions
- Keep DB access and schema migrations inside `libshogi` so other crates call into it rather than manipulate DB directly.
- Keep network/WebSocket quirks referenced from `libshogi/src/ref.py` when implementing listeners (e.g., keepalive/ping handling and version checks).
- Single source of truth for game state: store SFEN and ply in `moves` table to allow replay and idempotency.

Integration notes and external dependencies
- LiShogi WebSocket API: use `tokio-tungstenite` + careful keepalive. See `libshogi/src/ref.py` for expected message quirk handling.
- Database: intended for SQLite via Diesel; schema described in [README.md](README.md). Implement inserts with conflict handling.
- Rendering assets (SVG pieces) are external — the code expects to render using provided textures; keep rendering deterministic and testable.

What to prioritize for incremental work
- Implement socket manager and per-game listeners in `libshogi` (follow ref.py behavior).
- DB: implement idempotent move insert and game lifecycle (create/update/finish).
- Rendering tests: SFEN → image unit tests in `libshogi`.
- Discord commands in `bot` should be thin wrappers that call into `libshogi` for DB and rendering.

Do not ask for secrets (Discord tokens, private keys, etc.). If more context is needed, request only non-secret runtime details such as the database file path, the target deployment (Docker vs local), expected systemd/service names, or the names of environment variables used (but not their values).

-- End of instructions
