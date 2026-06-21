// 統合テスト用（各 `tests/test_*.rs` の先頭で `include!("include/helpers.rs");` する）
//
// エンコーダー用クレートでは `decoder_config_aac` / `encode_aac_packets` / `sine_pcm` が、
// デコーダー用では `encoder_config` が未使用になる。

const TEST_SAMPLE_RATE: u32 = 48000;
const TEST_CHANNELS: u8 = 2;
const AAC_FRAMES_PER_PACKET: usize = 1024;

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

/// 非無音のステレオ正弦波 PCM を生成する
#[allow(dead_code)]
fn sine_pcm(frames: usize) -> Vec<i16> {
    use std::f64::consts::PI;
    let mut pcm = Vec::with_capacity(frames * TEST_CHANNELS as usize);
    for i in 0..frames {
        // 1000 Hz の正弦波を 0.5 の振幅で生成する
        let sample = (0.5 * (2.0 * PI * 1000.0 * i as f64 / TEST_SAMPLE_RATE as f64).sin()
            * i16::MAX as f64)
            .round() as i16;
        pcm.push(sample);
        pcm.push(sample);
    }
    pcm
}

/// 非無音の AAC-LC パケットを複数個生成する
#[allow(dead_code)]
fn encode_aac_packets(packet_count: usize) -> Vec<Vec<u8>> {
    let mut encoder = shiguredo_audio_toolbox::Encoder::new(encoder_config(Some(128_000)))
        .expect("この環境では Encoder::new が成功しなければならない");
    let pcm = sine_pcm(packet_count * AAC_FRAMES_PER_PACKET);
    encoder.encode(&pcm).expect("正弦波 PCM のエンコード");
    encoder.finish().expect("エンコードの終了処理");

    let mut packets = Vec::new();
    while let Some(frame) = encoder.next_frame() {
        packets.push(frame.data);
    }
    packets
}
