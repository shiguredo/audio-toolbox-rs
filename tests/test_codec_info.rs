//! `codec_info` モジュールのテスト（`src/codec_info.rs` に対応）

use shiguredo_audio_toolbox::{AudioCodecType, supported_codecs};

#[test]
fn supported_codecs_has_expected_length_and_unique_codec_entries() {
    let codecs = supported_codecs();
    // `codec_types_for_supported_codecs` と一致する（Encoder/Decoder 表面の種別数）
    assert_eq!(codecs.len(), 3);
    for i in 0..codecs.len() {
        for j in (i + 1)..codecs.len() {
            assert_ne!(
                codecs[i].codec, codecs[j].codec,
                "supported_codecs must not contain duplicate AudioCodecType entries"
            );
        }
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
fn supported_codecs_opus_decoding_supported() {
    let codecs = supported_codecs();
    let opus = codecs
        .iter()
        .find(|c| c.codec == AudioCodecType::Opus)
        .expect("Opus entry");
    assert!(opus.decoding.supported);
}
