use engine::Engine;
use engine8::search::Engine8;

pub mod engine;
pub mod engine12;
pub mod engine6;
pub mod engine8;

// fn main() {
//     let mut pos = P8::default();
//     pos.set_sfen("rnbqkbnr/pppppppp/8/PP1PP1P1/6P1/KP6/2PPP2P/1NBQ1BNR w - 1")
//         .unwrap();
//     let pawns = pos.player_bb(Color::White) & &pos.type_bb(&PieceType::Pawn);
//     let chains = count_pawn_chains(pawns, &pos, Color::White);
//     dbg!(chains);
// }
//
fn main() {
    // let best_value = i32::MIN;
    // dbg!(best_value);
    // dbg!(best_value.max(250));

    let mut engine = Engine8::new();
    engine.uci_loop("4k3/4r3/8/8/6n1/4B1/5PPP/5BNK b - 1");
    // engine.uci_loop("10k1/12/10n1/12/12/12/12/12/10n1/12/9_._.P/8BBNK b - 1");
}
