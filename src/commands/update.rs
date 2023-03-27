use self_update::cargo_crate_version;

pub fn update() -> Result<(), anyhow::Error> {
    self_update::backends::github::Update::configure()
        .repo_owner("jasal82")
        .repo_name("metabuild")
        .bin_name("mb")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    Ok(())
}
