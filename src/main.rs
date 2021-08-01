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
    cmd: Option<Command>,

    /// Print succinct output when possible.
    #[clap(long, short, global = true)]
    quiet: bool,

    /// Print machine-readable output when possible.
    #[clap(long, global = true)]
    plain: bool,
}

#[derive(Clap)]
enum Command {
    /// Display the hero's status [default]
    #[clap(aliases=&["s", "status"], display_order=0)]
    Stat,

    /// Moves the hero to the supplied destination, potentially initiating battles along the way.
    #[clap(name = "cd", display_order = 1)]
    ChangeDir {
        /// Directory to move to.
        #[clap(default_value = "~")]
        destination: String,

        /// Attempt to avoid battles by running away.
        #[clap(long)]
        run: bool,

        /// Attempt to avoid battles by bribing the enemy.
        #[clap(long)]
        bribe: bool,

        /// Move the hero's to a different location without spawning enemies.
        /// Intended for scripts and shell integration.
        #[clap(short, long)]
        force: bool,
    },

    /// Inspect the directory contents, possibly finding treasure chests and hero tombstones.
    #[clap(name = "ls", display_order = 1)]
    Inspect,

    /// Buys an item from the shop.
    /// If name is omitted lists the items available for sale.
    #[clap(alias = "b", display_order = 2)]
    Buy { item: Option<String> },

    /// Uses an item from the inventory.
    #[clap(alias = "u", display_order = 3)]
    Use { item: Option<String> },

    /// Prints the quest todo list.
    #[clap(alias = "t", display_order = 4)]
    Todo,

    /// Resets the current game.
    Reset {
        /// Reset data files, losing cross-hero progress.
        #[clap(long)]
        hard: bool,
    },

    /// Change the character class.
    /// If name is omitted lists the available character classes.
    Class { name: Option<String> },

    /// Prints the hero's current location
    #[clap(name = "pwd")]
    PrintWorkDir,

    /// Potentially initiates a battle in the hero's current location.
    Battle {
        /// Attempt to avoid battles by running away.
        #[clap(long)]
        run: bool,

        /// Attempt to avoid battles by bribing the enemy.
        #[clap(long)]
        bribe: bool,
    },
}

fn main() {
    let mut exit_code = 0;

    let opts: Opts = Opts::parse();
    log::init(opts.quiet, opts.plain);

    // reset --hard is a special case, it needs to work when we
    // fail to deserialize the game data -- e.g. on backward
    // incompatible changes
    if let Some(Command::Reset { hard: true }) = opts.cmd {
        datafile::remove();
    }

    datafile::load_classes();

    let mut game = datafile::load().unwrap_or_else(|_| Game::new());

    // TODO move to run command pub fun
    match opts.cmd.unwrap_or(Command::Stat) {
        Command::Stat => log::status(&game),
        Command::ChangeDir {
            destination,
            run,
            bribe,
            force,
        } => {
            exit_code = command::change_dir(&mut game, &destination, run, bribe, force);
        }
        Command::Inspect => {
            game.inspect();
        }
        Command::Class { name } => command::class(&mut game, &name),
        Command::Battle { run, bribe } => {
            exit_code = command::battle(&mut game, run, bribe);
        }
        Command::PrintWorkDir => println!("{}", game.location.path_string()),
        Command::Reset { .. } => game.reset(),
        Command::Buy { item } => command::shop(&mut game, &item),
        Command::Use { item } => command::use_item(&mut game, &item),
        Command::Todo => {
            let (todo, done) = game.quests.list(&game);
            log::quest_list(&todo, &done);
        }
    }

    datafile::save(&game).unwrap();
    std::process::exit(exit_code);
}
