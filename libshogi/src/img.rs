use shogi_img::{pos2img, shogi_core};
use shogi_usi_parser::FromUsi;

pub fn save_img(sfen: String) -> String {
    let pos = shogi_core::PartialPosition::from_usi(format!("sfen {}", sfen).as_str())
        .expect("Should be able to parse LiShogi SFEN.");

    let hash_bytes = blake3::hash(sfen.as_bytes());
    let path = format!("img/{}.png", hash_bytes);

    if !std::path::Path::new(&path).exists() {
        pos2img(&pos)
            .save(path.as_str())
            .expect("Should be able to save image.");
    }

    path
}
