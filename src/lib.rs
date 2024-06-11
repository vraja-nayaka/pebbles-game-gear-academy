#![no_std]

use gstd::msg;
use pebbles_game_io::*;

static mut PEBBLES_GAME: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let init_message: PebblesInit = msg::load().expect("Can't load init message");
    let initial_state = GameState {
        pebbles_count: init_message.pebbles_count,
        max_pebbles_per_turn: init_message.max_pebbles_per_turn,
        pebbles_remaining: init_message.pebbles_count,
        difficulty: init_message.difficulty,
        first_player: Player::User,
        winner: None,
    };
    unsafe {
        PEBBLES_GAME = Some(initial_state);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Unable to decode PebblesAction");
    let mut game_state = unsafe { PEBBLES_GAME.take().expect("Game state is not initialized") };

    match action {
        PebblesAction::Turn(pebbles_taken) => {
            if pebbles_taken > game_state.max_pebbles_per_turn || pebbles_taken == 0 {
                panic!("Invalid number of pebbles taken");
            }

            if pebbles_taken > game_state.pebbles_remaining {
                panic!("Not enough pebbles remaining");
            }

            game_state.pebbles_remaining -= pebbles_taken;

            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::User);
                msg::reply(PebblesEvent::Won(Player::User), 0)
                    .expect("Failed to reply with Won event");
            } else {
                // Simple AI for Program
                let counter_pebbles_taken =
                    if game_state.pebbles_remaining <= game_state.max_pebbles_per_turn {
                        game_state.pebbles_remaining
                    } else {
                        (game_state.max_pebbles_per_turn / 2).max(1)
                    };

                game_state.pebbles_remaining -= counter_pebbles_taken;
                msg::reply(PebblesEvent::CounterTurn(counter_pebbles_taken), 0)
                    .expect("Failed to reply with CounterTurn event");

                if game_state.pebbles_remaining == 0 {
                    game_state.winner = Some(Player::Program);
                    msg::reply(PebblesEvent::Won(Player::Program), 0)
                        .expect("Failed to reply with Won event");
                }
            }
        }
        PebblesAction::GiveUp => {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0)
                .expect("Failed to reply with Won event");
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            game_state = GameState {
                pebbles_count,
                max_pebbles_per_turn,
                pebbles_remaining: pebbles_count,
                difficulty,
                first_player: Player::User,
                winner: None,
            };
        }
    }

    unsafe {
        PEBBLES_GAME = Some(game_state);
    }
}

#[no_mangle]
extern "C" fn state() {
    let game_state = unsafe { PEBBLES_GAME.clone().expect("Game state is not initialized") };
    msg::reply(game_state, 0).expect("Failed to share state");
}
