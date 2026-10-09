//! A cargo-subcommand utility for building and packaging rust binaries using `pebble-sys`.
#![allow(incomplete_features)]
pub mod build_project;
pub mod install;
pub mod template;

use clap::Parser;
use color_eyre::eyre::Result;

use crate::{build_project::BuildProject, install::Install, template::NewProject};

#[derive(Debug, clap::Parser)]
#[command(name = "cargo gneiss", about = "A build tool for Gneiss Pebble apps")]
struct Command {
	#[command(subcommand)]
	task: Task,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Task {
	New(NewProject),
	Build(BuildProject),
	Install(Install),
}

fn main() -> Result<()> {
	let hooks = color_eyre::config::HookBuilder::default();
	let hooks = if cfg!(debug_assertions) {
		hooks
	} else {
		hooks
			.display_location_section(false)
			.display_env_section(false)
	};
	hooks.install()?;
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.parse_default_env()
		.init();

	// When invoked as a cargo subcommand (`cargo gneiss …`), cargo passes the
	// subcommand name as argv[1]. Strip it so clap sees the *real* subcommand,
	// while still allowing `cargo-gneiss …` to be run directly for debugging.
	let args = std::env::args()
		.enumerate()
		.filter_map(|(i, a)| (!(i == 1 && a == "gneiss")).then_some(a));
	let cmd = Command::parse_from(args);

	match cmd.task {
		Task::New(project) => project.create(),
		Task::Build(project) => project.build(),
		Task::Install(install) => install.install(),
	}?;

	Ok(())
}
