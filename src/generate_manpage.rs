use man::prelude::*;

fn main() {
    // TODO(peap): Consider switching to clap_mangen.
    let app = git_global::get_clap_app();
    let name_and_email: Vec<&str> =
        app.p.meta.author.unwrap().split(" <").collect();
    let name = name_and_email[0];
    let email = name_and_email[1].strip_suffix(">").unwrap();
    let mut page = Manual::new(app.get_name())
        .about(app.p.meta.about.unwrap())
        .author(Author::new(name).email(email))
        .custom(Section::new("version").paragraph(&format!(
            "Crate version {}",
            app.p.meta.version.unwrap()
        )));
    for arg in app.p.global_args {
        page = page.flag(
            Flag::new()
                .short(&arg.s.short.unwrap().to_string())
                .long(arg.s.long.unwrap())
                .help(arg.b.help.unwrap()),
        );
    }
    let mut commands_section = Section::new("subcommands").paragraph(
        "The following subcommands are supported by git global; \
                        use git's global config to set your default choice.",
    );
    for cmd in app.p.subcommands {
        commands_section = commands_section.paragraph(&format!(
            "{}: {}",
            cmd.p.meta.name,
            cmd.p.meta.about.unwrap()
        ));
    }
    page = page.custom(commands_section);
    println!("{}", page.render());
}
