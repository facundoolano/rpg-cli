use game::Game;

mod character;
mod command;
mod datafile;
mod game;
mod item;
mod location;
mod log;
mod quest;
mod randomizer;

use anyhow::Result;
use clap::{crate_version, Parser};

/// Your filesystem as a dungeon!
#[derive(Parser)]
#[clap(version = crate_version!(), author = "Facundo Olano <facundo.olano@gmail.com>")]
struct Opts {
    #[clap(subcommand)]
    cmd: Option<command::Command>,

    /// Print succinct output when possible.
    #[clap(long, short, global = true)]
    quiet: bool,

    /// Print machine-readable output when possible.
    #[clap(long, global = true)]
    plain: bool,
}

fn main() {
    if let Err(err) = run_game() {
        // don't print a new line if error message is empty
        if !err.to_string().is_empty() {
            println!("{}", err);
        };

        std::process::exit(1);
    }
}

/// Loads or creates a new game, executes the received command and saves.
/// Inner errors are bubbled up.
fn run_game() -> Result<()> {
    let opts: Opts = Opts::parse();
    log::init(opts.quiet, opts.plain);
    datafile::load_classes();

    // reset --hard is a special case, it needs to work when we
    // fail to deserialize the game data -- e.g. on backward
    // incompatible changes
    if let Some(command::Command::Reset { hard: true }) = opts.cmd {
        datafile::remove();
    }

    let mut game = datafile::load()?.unwrap_or_else(Game::new);

    let result = command::run(opts.cmd, &mut game);

    // save the file regardless of the success of the command.
    // E.g. if the player dies it's an error / exit code 1
    // and that needs to be reflected in the game state.
    datafile::save(&game).unwrap();

    result
}
