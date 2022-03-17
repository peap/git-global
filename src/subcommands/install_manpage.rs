//! The `install-manpage` subcommand: attempts to install a man page.

use crate::config::Config;
use crate::errors::Result;
use crate::report::Report;

// TODO(peap): Add option to just generate the file for the user to stick somewhere?

/// Attempts to install git-global's man page to the relevant directory.
/// This is a work-around to not maintaining distribution-specific packages
/// and Cargo not providing this functionality for crates.
pub fn execute(mut config: Config) -> Result<Report> {
    let repos = config.get_repos();
    let mut report = Report::new(&repos);
    report.add_message("This feature is a work-in-progress.".to_string());
    report.add_message(
        "In the meantime, you can find the manpage at \
         https://raw.githubusercontent.com/peap/git-global/master/doc/git-global.1".to_string()
    );
    if let Some(manpage_file) = config.manpage_file {
        report.add_message(format!(
            "...would write file to {}",
            manpage_file.display()
        ));
    } else {
        report.add_message("...not sure where to put it!".to_string());
    }
    Ok(report)
}
