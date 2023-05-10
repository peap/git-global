use man::prelude::*;

fn main() {
    // TODO(peap): Consider switching to clap_mangen.
    let app = git_global::get_clap_app();
    let name_and_email: Vec<&str> =
        app.get_author().unwrap().split(" <").collect();
    let name = name_and_email[0];
    let email = name_and_email[1].strip_suffix(">").unwrap();
    let mut page = Manual::new(app.get_name())
        .about(app.get_about().unwrap().to_string())
        .author(Author::new(name).email(email))
        .custom(Section::new("version").paragraph(&format!(
            "Crate version {}",
            app.get_version().unwrap()
        )));
    for arg in app.get_arguments() {
        page = page.flag(
            Flag::new()
                .short(&arg.get_short().unwrap().to_string())
                .long(arg.get_long().unwrap())
                .help(&arg.get_help().unwrap().to_string()),
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
