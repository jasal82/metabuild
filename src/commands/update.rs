use colored::Colorize;
use self_update::cargo_crate_version;

pub fn update(gitlab_token: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(token) = gitlab_token {
        let status = self_update::backends::gitlab::Update::configure()
            .with_host("https://gitlab.sickcn.net")
            .repo_owner("asaljo")
            .repo_name("metabuild")
            .auth_token(token)
            .bin_name("mb")
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;
        println!("Update status: {}", status.version());
        Ok(())
    } else {
        println!("Please set a personal access token for https://gitlab.sickcn.net first");
        println!(
            "by running {}.",
            "mb config set --global gitlab_token <token>".bright_green()
        );
        Err("Failed to retrieve list of releases from Gitlab".into())
    }
}
