//! `Decoder` の単体テスト（`src/lib.rs` のデコーダー部分に対応）
//!
//! 圧縮バイト列の任意入力でのクラッシュ耐性は cargo-fuzz で扱う。

use shiguredo_audio_toolbox::{Decoder, DecoderCodec, DecoderConfig};

fn decoder_aac_stereo_48k() -> Decoder {
    Decoder::new(DecoderConfig {
        codec: DecoderCodec::AacLc,
        input_sample_rate: 48000,
        input_channels: 2,
    })
    .expect("Decoder::new must succeed on this platform")
}

/// 空のスライスを `decode` してもバッファは空のままなので、直後に別の `decode` が可能である。
#[test]
fn decode_empty_then_decode_non_empty_does_not_error() {
    let mut d = decoder_aac_stereo_48k();
    d.decode(&[]).expect("empty decode");
    d.decode(&[0x00, 0x01, 0x02])
        .expect("second decode after empty");
    assert!(
        d.decode(&[0xff]).is_err(),
        "third decode without next_frame must fail"
    );
}

/// 未消費のパケットがあるとき、2 回目の `decode` は失敗する。
#[test]
fn decode_second_without_next_frame_returns_error() {
    let mut d = decoder_aac_stereo_48k();
    d.decode(&[0u8; 8]).expect("first decode");
    let err = d.decode(&[0u8; 8]).expect_err("second decode must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("previous packet not consumed") && msg.contains("status=-50"),
        "unexpected error: {msg}"
    );
}
