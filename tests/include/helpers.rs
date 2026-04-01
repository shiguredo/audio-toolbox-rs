// 統合テスト用（各 `tests/test_*.rs` の先頭で `include!("include/helpers.rs");` する）
//
// エンコーダー用クレートでは `decoder_config_aac` が、デコーダー用では `encoder_config` が未使用になる。

const TEST_SAMPLE_RATE: u32 = 48000;
const TEST_CHANNELS: u8 = 2;

#[allow(dead_code)]
fn encoder_config(bitrate: Option<u32>) -> shiguredo_audio_toolbox::EncoderConfig {
    shiguredo_audio_toolbox::EncoderConfig {
        codec: shiguredo_audio_toolbox::EncoderCodec::AacLc,
        sample_rate: TEST_SAMPLE_RATE,
        channels: TEST_CHANNELS,
        bitrate,
        bitrate_control_mode: None,
        codec_quality: None,
        vbr_quality: None,
    }
}

#[allow(dead_code)]
fn decoder_config_aac() -> shiguredo_audio_toolbox::DecoderConfig {
    shiguredo_audio_toolbox::DecoderConfig {
        codec: shiguredo_audio_toolbox::DecoderCodec::AacLc,
        input_sample_rate: TEST_SAMPLE_RATE,
        input_channels: TEST_CHANNELS,
    }
}
