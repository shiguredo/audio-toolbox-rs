//! `codec_info` モジュールのテスト（`src/codec_info.rs` に対応）

use shiguredo_audio_toolbox::{AudioCodecType, supported_codecs};

#[test]
fn supported_codecs_contains_each_codec_type() {
    let codecs = supported_codecs();
    assert_eq!(codecs.len(), AudioCodecType::all().len());

    for expected in AudioCodecType::all() {
        assert!(
            codecs.iter().any(|c| c.codec == *expected),
            "supported_codecs missing {expected:?}"
        );
    }
}

#[test]
fn supported_codecs_aac_lc_decode_and_encode() {
    let codecs = supported_codecs();
    let aac_lc = codecs
        .iter()
        .find(|c| c.codec == AudioCodecType::AacLc)
        .expect("AAC-LC entry");
    assert!(aac_lc.decoding.supported);
    assert!(aac_lc.encoding.supported);
    assert!(!aac_lc.encoding.bitrate_control_modes.is_empty());
}

#[test]
fn supported_codecs_mp3_decode_only_typically() {
    let codecs = supported_codecs();
    let mp3 = codecs
        .iter()
        .find(|c| c.codec == AudioCodecType::Mp3)
        .expect("MP3 entry");
    assert!(mp3.decoding.supported);
    assert!(!mp3.encoding.supported);
}

#[test]
fn supported_codecs_alac_lossless_entry() {
    let codecs = supported_codecs();
    let alac = codecs
        .iter()
        .find(|c| c.codec == AudioCodecType::Alac)
        .expect("ALAC entry");
    assert!(alac.decoding.supported);
    assert!(alac.encoding.supported);
}
