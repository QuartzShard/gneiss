use std::{
    io::BufReader,
    process::{Command, Stdio},
};

use cargo_metadata::{Message, MetadataCommand, camino::Utf8PathBuf};
use color_eyre::eyre::{OptionExt, Result, bail};
use object::{Object, ObjectSection};

#[derive(Debug, Clone, clap::Parser)]
/// Build your Pebble app project
pub struct BuildProject {}

impl BuildProject {
    pub fn build(&self) -> Result<()> {
        let metadata = MetadataCommand::new().exec()?;
        let pkg_id = &metadata.root_package().ok_or_eyre("No root package")?.id;
        let mut build = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--message-format=json-render-diagnostics")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let stdout = build.stdout.take().expect("piped");
        let mut elf: Option<Utf8PathBuf> = None;
        let mut build_ok = false;
        for msg in Message::parse_stream(BufReader::new(stdout)) {
            match msg? {
                Message::CompilerArtifact(artifact) if artifact.package_id == *pkg_id => {
                    log::trace!("{:#?}", artifact);
                    elf = artifact.executable;
                }
                Message::BuildFinished(build_finished) => build_ok = build_finished.success,
                _ => (),
            };
        }
        if !build.wait()?.success() || !build_ok {
            bail!("cargo build failed")
        }
        let elf = elf.ok_or_eyre("No executable - make sure you're building a [[bin]]")?;
        log::debug!("ELF output at {elf}");
        let elf = std::fs::read(elf)?;
        let elf: object::File<'_> = object::File::parse(&*elf)?;
        let pbl_header = elf
            .sections()
            .inspect(|sec| log::debug!("{:?}", sec.name()))
            .find(|section| section.name().unwrap_or("") == ".pbl_header")
            .ok_or_eyre(".pbl_header missing")?;

        Ok(())
    }
}
