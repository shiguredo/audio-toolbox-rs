//! `Decoder` の単体テスト（`src/lib.rs` のデコーダー部分に対応）
//!
//! 圧縮バイト列の任意入力でのクラッシュ耐性は cargo-fuzz で扱う。
//! 共有定数・ファクトリは `include!("include/helpers.rs");` で取り込む。

include!("include/helpers.rs");

use shiguredo_audio_toolbox::{Decoder, DecoderCodec, DecoderConfig};

fn decoder_aac_stereo_48k() -> Decoder {
    Decoder::new(decoder_config_aac()).expect("Decoder::new must succeed on this platform")
}

#[test]
fn decoder_new_rejects_zero_sample_rate() {
    let r = Decoder::new(DecoderConfig {
        codec: DecoderCodec::AacLc,
        input_sample_rate: 0,
        input_channels: TEST_CHANNELS,
    });
    assert!(r.is_err());
    assert!(
        r.unwrap_err()
            .to_string()
            .contains("Decoder::new(input_sample_rate)")
    );
}

#[test]
fn decoder_new_rejects_zero_channels() {
    let r = Decoder::new(DecoderConfig {
        codec: DecoderCodec::AacLc,
        input_sample_rate: TEST_SAMPLE_RATE,
        input_channels: 0,
    });
    assert!(r.is_err());
    assert!(
        r.unwrap_err()
            .to_string()
            .contains("Decoder::new(input_channels)")
    );
}

#[test]
fn init_decoder_mp3() {
    let result = Decoder::new(DecoderConfig {
        codec: DecoderCodec::Mp3,
        input_sample_rate: 44100,
        input_channels: 2,
    });
    assert!(result.is_ok());
}

#[test]
fn init_decoder_opus() {
    let result = Decoder::new(DecoderConfig {
        codec: DecoderCodec::Opus,
        input_sample_rate: 48000,
        input_channels: 2,
    });
    assert!(result.is_ok());
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

#[test]
fn decode_finish_then_next_frame_returns_none_without_input() {
    let mut d = decoder_aac_stereo_48k();
    d.finish().expect("finish");
    assert!(d.next_frame().expect("next_frame").is_none());
    assert!(d.next_frame().expect("next_frame").is_none());
}
