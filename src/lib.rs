#![no_std]

use gstd::{exec, msg};
use pebbles_game_io::*;

static mut PEBBLES_GAME: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let init_message: PebblesInit = msg::load().expect("Can't load init message");
    let first_player = get_first_player();

    let pebbles_remaining = get_init_pebbles_remain(
        init_message.pebbles_count,
        init_message.max_pebbles_per_turn,
        first_player.clone(),
        init_message.difficulty.clone(),
    );

    let initial_state = GameState {
        pebbles_count: init_message.pebbles_count,
        max_pebbles_per_turn: init_message.max_pebbles_per_turn,
        pebbles_remaining,
        difficulty: init_message.difficulty,
        first_player,
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
                let counter_pebbles_taken = get_contract_pebbles_taken(
                    game_state.pebbles_remaining,
                    game_state.max_pebbles_per_turn,
                    game_state.difficulty,
                );

                game_state.pebbles_remaining -= counter_pebbles_taken;

                if game_state.pebbles_remaining == 0 {
                    game_state.winner = Some(Player::Program);
                    msg::reply(PebblesEvent::Won(Player::Program), 0)
                        .expect("Failed to reply with Won event");
                } else {
                    msg::reply(PebblesEvent::CounterTurn(counter_pebbles_taken), 0)
                        .expect("Failed to reply with CounterTurn event");
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
            let first_player = get_first_player();
            let pebbles_remaining = get_init_pebbles_remain(
                pebbles_count,
                max_pebbles_per_turn,
                first_player.clone(),
                difficulty.clone(),
            );

            game_state = GameState {
                pebbles_count,
                max_pebbles_per_turn,
                pebbles_remaining,
                difficulty,
                first_player,
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

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

fn get_contract_pebbles_taken(
    pebbles_remaining: u32,
    max_pebbles_per_turn: u32,
    difficulty: DifficultyLevel,
) -> u32 {
    match difficulty {
        DifficultyLevel::Easy => {
            // Choose a random number of pebbles to take (between 1 and max_pebbles_per_turn)
            let random_number = get_random_u32();
            (random_number % max_pebbles_per_turn + 1).min(pebbles_remaining)
        }
        DifficultyLevel::Hard => {
            // Implementing a winning strategy for hard difficulty
            let optimal_pebbles_taken = pebbles_remaining % (max_pebbles_per_turn + 1);
            if optimal_pebbles_taken == 0 {
                1
            } else {
                optimal_pebbles_taken
            }
        }
    }
}

fn get_first_player() -> Player {
    let random_number = get_random_u32();
    let first_player = if random_number % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };
    first_player
}

fn get_init_pebbles_remain(
    pebbles_count: u32,
    max_pebbles_per_turn: u32,
    first_player: Player,
    difficulty: DifficultyLevel,
) -> u32 {
    let mut pebbles_remaining = pebbles_count;

    if first_player == Player::Program {
        let counter_pebbles_taken =
            get_contract_pebbles_taken(pebbles_count, max_pebbles_per_turn, difficulty);

        pebbles_remaining -= counter_pebbles_taken;
        msg::reply(PebblesEvent::CounterTurn(counter_pebbles_taken), 0)
            .expect("Failed to reply with CounterTurn event");
    }

    pebbles_remaining
}
