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
	Ls {
		filter: Option<String>
	},
	Init {
		name: String
	},
	Delete {
		name: String
	},
	Publish {
		name: String
	},
	Fetch {
		name: String
	},
	Which {
		name: String
	},
	Exec {
		#[arg(short = 'n')]
		name: Option<String>,
		script: String
	},
	Setup,
	Add {
		name: String
	},
	#[command(alias = "rm")]
	Remove {
		name: String
	},
	Upgrade,
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
