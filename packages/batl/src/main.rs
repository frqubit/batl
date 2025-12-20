use clap::{Parser, Subcommand, Args};

mod commands;
mod output;
mod utils;

#[derive(Parser)]
#[command(name = "batl")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "The multi-repo development tool")]
struct Cli {
	#[command(subcommand)]
	subcmd: SubCommand
}

#[derive(Subcommand)]
enum SubCommand {
	#[command(about = "[DEPRECATED 0.3.0] Old repository command aliases")]
	Repository(SubCmdArgs<commands::repository::Commands>),
	#[command(about = "Lists present battalion repositories")]
	Ls {
		filter: Option<String>
	},
	#[command(about = "Creates a new battalion repository")]
	Init {
		name: String
	},
	#[command(about = "Deletes a battalion repository (be careful!)")]
	Delete {
		name: String
	},
	#[command(about = "Publishes a repository")]
	Publish {
		name: String
	},
	#[command(about = "Fetches a repository from the battalion registry")]
	Fetch {
		name: String
	},
	#[command(about = "Gets the system path to a repository")]
	Which {
		name: String
	},
	#[command(about = "Executes a command on a repository")]
	Exec {
		#[arg(short = 'n')]
		name: Option<String>,
		script: String
	},
	#[command(about = "Sets up battalion")]
	Setup,
	#[command(about = "Adds a dependency")]
	Add {
		name: String
	},
	#[command(about = "Removes a dependency")]
	#[command(alias = "rm")]
	Remove {
		name: String
	},
	#[command(about = "Upgrades the installed battalion to the newest version")]
	Upgrade,
	#[command(about = "Adds an API key")]
	Auth
}

#[derive(Args)]
struct SubCmdArgs<T: Subcommand> {
	#[command(subcommand)]
	subcmd: T
}


fn main() {
	let cli = Cli::parse();

	let result = match cli.subcmd {
		SubCommand::Repository(args) => commands::repository::run(args.subcmd),
		SubCommand::Setup => commands::cmd_setup(),
		SubCommand::Add { name } => commands::cmd_add(name),
		SubCommand::Remove { name } => commands::cmd_remove(name),
		SubCommand::Upgrade => commands::cmd_upgrade(),
		SubCommand::Auth => commands::cmd_auth(),
		SubCommand::Ls { filter } => commands::cmd_ls(filter),
		SubCommand::Init { name } => commands::cmd_init(name),
		SubCommand::Delete { name } => commands::cmd_delete(name),
		SubCommand::Publish { name } => commands::cmd_publish(name),
		SubCommand::Fetch { name } => commands::cmd_fetch(name),
		SubCommand::Exec { name, script } => commands::cmd_exec(name, script),
		SubCommand::Which { name } => commands::cmd_which(name)
	};

	if let Err(err) = result {
		output::error(err.to_string().as_str());
		std::process::exit(1);
	}
}
