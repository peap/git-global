#[test]
fn verify_cli() {
    let app = git_global::get_clap_app();
    app.debug_assert();
}
