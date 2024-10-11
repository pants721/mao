use mao_core::*;

fn main() {
    let mut game = Game::new(2, vec!["lucas", "lucas2"]);
    game.draw_card("lucas2");
    dbg!(&game);
}
