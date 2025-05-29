use mseq_tracks::index::load_from_file;

#[test]
fn load_from_index() {
    let tracks = load_from_file("tests/res/index.toml").unwrap();
}
