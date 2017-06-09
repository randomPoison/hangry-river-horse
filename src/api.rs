use broadcast::*;
use game::*;
use rocket::State;
use rocket_contrib::JSON;
use std::sync::*;

#[derive(Debug, Serialize)]
pub struct RegisterPlayerResponse {
    id: PlayerId,
}

/// Generates a `PlayerId` for a new player.
// TODO: Allow players to specify a username when registering.
#[get("/register-player")]
pub fn register_player(
    player_id_generator: State<PlayerIdGenerator>,
    scoreboard: State<Mutex<Scoreboard>>,
    broadcaster: State<HostBroadcaster>,
) -> JSON<RegisterPlayerResponse>
{
    let player_id = player_id_generator.next_id();

    // Add the player to the scoreboard.
    {
        let mut scoreboard = scoreboard.lock().expect("Scoreboard mutex was poisoned");
        let old = scoreboard.insert(player_id, 0);
        assert_eq!(None, old, "Player ID was registered twice");
    }

    // Broadcast to all hosts that a new player has joined.
    broadcaster.send(HostBroadcast::PlayerRegistered(player_id));

    // Respond to the client.
    JSON(RegisterPlayerResponse {
        id: player_id,
    })
}

#[derive(Debug, Deserialize)]
pub struct FeedPlayerRequest {
    player: PlayerId,
}

#[derive(Debug, Serialize)]
pub struct FeedPlayerResponse {
    score: usize,
}

#[post("/feed-me", format = "application/json", data = "<payload>")]
pub fn feed_player(
    payload: JSON<FeedPlayerRequest>,
    scoreboard: State<Mutex<Scoreboard>>,
    broadcaster: State<HostBroadcaster>,
) -> JSON<FeedPlayerResponse>
{
    let payload = payload.into_inner();
    let player_id = payload.player;

    // Add 1 to the player's score, returning the new score.
    let score = {
        let mut scoreboard = scoreboard.lock().expect("Scoreboard mutex was poisoned");

        // TODO: Return a proper error code if the player isn't registered, and also maybe don't
        // poison the mutex while we're at it.
        let score = scoreboard
            .get_mut(&player_id)
            .expect("Tried to add score to a nonexistent player");
        *score += 1;
        *score
    };

    // Broadcast the new score to all hosts.
    broadcaster.send(HostBroadcast::PlayerScore {
        player: player_id,
        score: score,
    });

    // Respond to the client.
    JSON(FeedPlayerResponse {
        score: score,
    })
}
