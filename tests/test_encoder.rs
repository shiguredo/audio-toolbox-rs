//! `Encoder` の単体・統合テスト（`src/lib.rs` のエンコーダー部分に対応）
//! 共有定数・ファクトリは `include!("include/helpers.rs");` で取り込む。

include!("include/helpers.rs");

use shiguredo_audio_toolbox::{
    BitRateControlMode, CodecQuality, Encoder, EncoderCodec, EncoderConfig,
};

#[test]
fn encoder_new_rejects_zero_sample_rate() {
    let r = Encoder::new(EncoderConfig {
        codec: EncoderCodec::AacLc,
        sample_rate: 0,
        channels: TEST_CHANNELS,
        bitrate: None,
        bitrate_control_mode: None,
        codec_quality: None,
        vbr_quality: None,
    });
    assert!(r.is_err());
    let msg = r.unwrap_err().to_string();
    assert!(msg.contains("Encoder::new(sample_rate)"));
}

#[test]
fn encoder_new_rejects_zero_channels() {
    let r = Encoder::new(EncoderConfig {
        codec: EncoderCodec::AacLc,
        sample_rate: TEST_SAMPLE_RATE,
        channels: 0,
        bitrate: None,
        bitrate_control_mode: None,
        codec_quality: None,
        vbr_quality: None,
    });
    assert!(r.is_err());
    let msg = r.unwrap_err().to_string();
    assert!(msg.contains("Encoder::new(channels)"));
}

#[test]
fn encoder_new_rejects_invalid_bitrate() {
    assert!(Encoder::new(encoder_config(Some(1_000))).is_err());
}

#[test]
fn encoder_new_accepts_default_bitrate() {
    assert!(Encoder::new(encoder_config(None)).is_ok());
}

#[test]
fn encoder_new_accepts_each_bitrate_control_mode() {
    for mode in [
        BitRateControlMode::Constant,
        BitRateControlMode::LongTermAverage,
        BitRateControlMode::VariableConstrained,
        BitRateControlMode::Variable,
    ] {
        let r = Encoder::new(EncoderConfig {
            codec: EncoderCodec::AacLc,
            sample_rate: TEST_SAMPLE_RATE,
            channels: TEST_CHANNELS,
            bitrate: Some(128_000),
            bitrate_control_mode: Some(mode),
            codec_quality: None,
            vbr_quality: None,
        });
        assert!(
            r.is_ok(),
            "Encoder::new with BitRateControlMode::{mode:?} failed: {:?}",
            r.err()
        );
    }
}

#[test]
fn encoder_new_accepts_each_codec_quality() {
    for quality in [
        CodecQuality::Min,
        CodecQuality::Low,
        CodecQuality::Medium,
        CodecQuality::High,
        CodecQuality::Max,
    ] {
        let r = Encoder::new(EncoderConfig {
            codec: EncoderCodec::AacLc,
            sample_rate: TEST_SAMPLE_RATE,
            channels: TEST_CHANNELS,
            bitrate: Some(128_000),
            bitrate_control_mode: None,
            codec_quality: Some(quality),
            vbr_quality: None,
        });
        assert!(
            r.is_ok(),
            "Encoder::new with CodecQuality::{quality:?} failed: {:?}",
            r.err()
        );
    }
}

#[test]
fn encoder_next_frame_none_when_no_encoded_data() {
    let mut enc = Encoder::new(encoder_config(Some(128_000))).expect("encoder");
    assert!(enc.next_frame().is_none());
}

#[test]
fn encoder_finish_on_empty_pcm_does_not_panic() {
    let mut enc = Encoder::new(encoder_config(Some(128_000))).expect("encoder");
    enc.finish().expect("finish");
    let _ = enc.next_frame();
}
