//! A cargo-subcommand utility for building and packaging rust binaries using `pebble-sys`.
#![allow(incomplete_features)]
pub mod build_project;
pub mod template;

use clap::Parser;
use color_eyre::eyre::Result;

use crate::{build_project::BuildProject, template::NewProject};

#[derive(Debug, clap::Parser)]
#[command(name = "cargo pebble", about = "A build tool for Pebble apps")]
struct Command {
	#[command(subcommand)]
	task: Task,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Task {
	New(NewProject),
	Build(BuildProject),
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

	// When invoked as a cargo subcommand (`cargo pebble …`), cargo passes the
	// subcommand name as argv[1]. Strip it so clap sees the *real* subcommand,
	// while still allowing `cargo-pebble …` to be run directly for debugging.
	let args = std::env::args()
		.enumerate()
		.filter_map(|(i, a)| (!(i == 1 && a == "pebble")).then_some(a));
	let cmd = Command::parse_from(args);

	match cmd.task {
		Task::New(project) => project.create(),
		Task::Build(project) => project.build(),
	}?;

	Ok(())
}
