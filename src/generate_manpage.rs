use man::prelude::*;

fn main() {
    // TODO(peap): Consider switching to clap_mangen.
    let app = git_global::get_clap_app();
    let name_and_email: Vec<&str> =
        app.get_author().unwrap().split(" <").collect();
    let name = name_and_email[0];
    let email = name_and_email[1].strip_suffix(">").unwrap();
    let mut page = Manual::new(app.get_name())
        .about(app.get_about().unwrap())
        .author(Author::new(name).email(email))
        .custom(Section::new("version").paragraph(&format!(
            "Crate version {}",
            app.get_version().unwrap()
        )));
    for arg in app.get_arguments() {
        if !arg.is_global_set() {
            continue;
        }
        let arg_name = arg.get_long().unwrap();
        if arg_name == "help" || arg_name == "version" {
            // Skip clap's built-in global flags.
            continue;
        }
        page = page.flag(
            Flag::new()
                .short(&arg.get_short().unwrap().to_string())
                .long(arg.get_long().unwrap())
                .help(arg.get_help().unwrap()),
        );
    }
    let mut commands_section = Section::new("subcommands").paragraph(
        "The following subcommands are supported by git global; \
                        use git's global config to set your default choice.",
    );
    for cmd in app.get_subcommands() {
        commands_section = commands_section.paragraph(&format!(
            "{}: {}",
            cmd.get_name(),
            cmd.get_about().unwrap()
        ));
    }
    page = page.custom(commands_section);
    println!("{}", page.render());
}
