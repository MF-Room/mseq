use mseq_core::Track;
use mseq_tracks::index::load_from_file;

#[test]
fn load_from_index() {
    let tracks = load_from_file("tests/res/index.toml").unwrap();
    assert!(tracks.len() == 4);
    for (t, n) in tracks {
        match n.as_str() {
            "tests/res/midi.mid" => {
                assert!(t.get_name().eq("midi"))
            }
            "tests/res/acid.csv" => {
                assert!(t.get_name().eq("acid"))
            }
            "tests/res/div.csv" => {
                assert!(t.get_name().eq("div"))
            }
            "tests/res/arp.csv" => {
                assert!(t.get_name().eq("arp"))
            }
            _ => {
                panic!("Wrong file name: {n}")
            }
        }
    }
}
