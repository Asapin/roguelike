use rltk::RGB;

#[derive(PartialEq)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut rltk::Rltk) -> GameOverResult {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Your journey has ended!",
    );
    ctx.print_color_centered(
        17,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "One day, we'll tell you all about how you did.",
    );
    ctx.print_color_centered(
        18,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "This day, sadly, is not in this chapter...",
    );
    ctx.print_color_centered(
        20,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        "Press any key to return to the menu.",
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}
