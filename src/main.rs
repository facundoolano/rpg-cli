use game::Game;

mod character;
mod command;
mod datafile;
mod event;
mod game;
mod item;
mod location;
mod log;
mod quest;
mod randomizer;

use clap::{crate_version, AppSettings, Clap};

/// Your filesystem as a dungeon!
#[derive(Clap)]
#[clap(global_setting = AppSettings::ColoredHelp)]
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
    let opts: Opts = Opts::parse();
    log::init(opts.quiet, opts.plain);

    // reset --hard is a special case, it needs to work when we
    // fail to deserialize the game data -- e.g. on backward
    // incompatible changes
    if let Some(command::Command::Reset { hard: true }) = opts.cmd {
        datafile::remove();
    }

    datafile::load_classes();

    let mut game = datafile::load().unwrap_or_else(|_| Game::new());

    let exit_code = command::run(opts.cmd, &mut game);

    datafile::save(&game).unwrap();
    std::process::exit(exit_code);
}
