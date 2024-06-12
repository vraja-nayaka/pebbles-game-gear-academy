use gtest::{Program, System};
use pebbles_game_io::*;

#[test]
fn test() {
    let system = System::new();
    let program = Program::current(&system);
    let pid = program.id();
    let sender_id = 100;

    // Initialization of the game
    let init_message = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 10,
        max_pebbles_per_turn: 4,
    };

    program.send(sender_id, init_message);

    // Check the initial state of the game
    let state: GameState = program
        .read_state(pid)
        .expect("Failed to get the initial state of the game");

    assert!(state.pebbles_remaining <= 10);
    assert_eq!(state.pebbles_count, 10);
    assert_eq!(state.max_pebbles_per_turn, 4);
    assert_eq!(state.difficulty, DifficultyLevel::Easy);
    assert_eq!(state.winner, None::<Player>);

    // Player's (User) turn
    program.send(sender_id, PebblesAction::Turn(2));

    // Check the state after user's turn
    let state: GameState = program
        .read_state(pid)
        .expect("Failed to get the state of the game after user's turn");

    assert!(state.pebbles_remaining < 8);

    // Player gives up
    program.send(sender_id, PebblesAction::GiveUp);

    // Check the state after player gives up
    let state: GameState = program
        .read_state(pid)
        .expect("Failed to get the state of the game after giving up");

    assert_eq!(state.winner, Some(Player::Program));

    // Restart the game
    let restart_message = PebblesAction::Restart {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 15,
        max_pebbles_per_turn: 10,
    };

    program.send(sender_id, restart_message);

    // Check the state after restart
    let state: GameState = program
        .read_state(pid)
        .expect("Failed to get the state of the game after restart");

    assert!(state.pebbles_remaining <= 15);
    assert_eq!(state.pebbles_count, 15);
    assert_eq!(state.max_pebbles_per_turn, 10);
    assert_eq!(state.difficulty, DifficultyLevel::Hard);
    assert_eq!(state.winner, None);
}
