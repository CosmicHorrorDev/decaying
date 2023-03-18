use anyhow::Context;

mod config;

// TODO: anyhow
// TODO: only delete files on launch since we don't want to delete files while someone may still be
//       using them
fn main() -> anyhow::Result<()> {
    anyhow::ensure!(
        std::env::args_os().count() == 1,
        "`vanishing` doesn't take any command line args"
    );

    let config_path = dirs::config_dir()
        .context("Failed locating config dir")?
        .join("vanishing")
        .join("config.toml");
    let config = config::try_read(&config_path)?;

    println!("{config:#?}");

    Ok(())
}
