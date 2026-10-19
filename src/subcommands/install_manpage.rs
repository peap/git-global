//! The `install-manpage` subcommand: installs the man page.

use std::fs::{File, create_dir_all};
use std::path::PathBuf;

use crate::config::Config;
use crate::errors::Result;
use crate::report::Report;

/// Installs git-global's man page to the configured directory or a custom path.
pub fn execute(config: Config, dest: Option<PathBuf>) -> Result<Report> {
    let mut report = Report::new(&[]);
    let target = dest.or(config.manpage_file);

    if let Some(target_path) = target {
        if let Some(parent) = target_path.parent()
            && !parent.as_os_str().is_empty()
        {
            create_dir_all(parent)?;
        }
        let mut file = File::create(&target_path)?;
        let app = crate::cli::get_clap_app();
        clap_mangen::Man::new(app).render(&mut file)?;
        report.add_message(format!(
            "Installed manpage to {}",
            target_path.display()
        ));
    } else {
        report.add_message(
            "Could not determine where to install the manpage on this system."
                .to_string(),
        );
    }
    Ok(report)
}
