use std::process;

mod config {
    mod paths {
        use std::path::{Path, PathBuf};
        use anyhow::Context;
        use directories::ProjectDirs;
        use once_cell::sync::OnceCell;

        const QUALIFIER: &str = "com";
        const ORGANIZATION: &str = "ALinuxPerson";
        const APPLICATION: &str = "synthtext";
        static PROJECT_DIRS: OnceCell<ProjectDirs> = OnceCell::new();
        static DIRECTORY: OnceCell<&Path> = OnceCell::new();
        static API_KEY: OnceCell<PathBuf> = OnceCell::new();

        pub fn initialize() -> anyhow::Result<()> {
            if PROJECT_DIRS.get().is_none() {
                let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
                    .context("failed to initialize project directories")?;
                let _ = PROJECT_DIRS.set(project_dirs);
            }

            Ok(())
        }

        fn project_dirs() -> &'static ProjectDirs {
            PROJECT_DIRS.get().expect("project directories not initialized")
        }

        pub fn directory() -> &'static Path {
            DIRECTORY.get_or_init(|| project_dirs().config_dir())
        }

        pub fn api_key() -> &'static Path {
            API_KEY.get_or_init(|| directory().join("api_key"))
        }
    }
}
mod args {
    use clap::Parser;

    #[derive(Debug, Parser)]
    pub struct SynthText {
        pub api_key: Option<String>,

        #[clap(subcommand)]
        pub action: SynthTextAction,
    }

    #[derive(Debug, Parser)]
    pub enum SynthTextAction {

    }
}

fn main() {
    fn inner() -> anyhow::Result<()> { Ok(()) }

    let exit_code = match inner() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:#}", err);
            1
        }
    };

    process::exit(exit_code)
}
