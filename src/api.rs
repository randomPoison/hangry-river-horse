use rocket_contrib::{JSON, Value};

use std::sync::atomic::*;

#[derive(Debug, Serialize)]
struct RegisterPlayerResponse {
    id: usize,
}

#[get("/register-player")]
fn register_player() -> JSON<RegisterPlayerResponse> {
    static PLAYER_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

    let next_id = PLAYER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    // TODO: Update game state to reflect the registered player.

    JSON(RegisterPlayerResponse {
        id: next_id,
    })
}
