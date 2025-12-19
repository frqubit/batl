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
	Repository(SubCmdArgs<commands::repository::Commands>),
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
		SubCommand::Repository(args) => commands::repository::run(args.subcmd),
		SubCommand::Setup => commands::cmd_setup(),
		SubCommand::Add { name } => commands::cmd_add(name),
		SubCommand::Remove { name } => commands::cmd_remove(name),
		SubCommand::Upgrade => commands::cmd_upgrade(),
		SubCommand::Auth => commands::cmd_auth()
	};

	if let Err(err) = result {
		output::error(err.to_string().as_str());
		std::process::exit(1);
	}
}
