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

/// 複数の AAC-LC パケットを連続投入した場合、各パケットが 1 回ずつデコードされ、
/// 同じパケットが 1 回の next_frame() 内で複数回提供されないことを検証する。
#[test]
fn decode_multiple_aac_packets_no_duplicate_feeds() {
    // エンコーダーに渡す 1024 フレーム単位のブロック数。
    // エンコーダーはプライミング等のため、これより多くの圧縮パケットを生成することがある。
    let input_blocks = 3;
    let packets = encode_aac_packets(input_blocks);
    assert!(
        !packets.is_empty(),
        "エンコーダーは 1 パケット以上生成しなければならない"
    );

    let mut decoder = decoder_aac_stereo_48k();
    let mut total_frames = 0usize;

    for (i, packet) in packets.iter().enumerate() {
        decoder
            .decode(packet)
            .unwrap_or_else(|e| panic!("パケット {i} の decode に失敗した: {e}"));

        // `Ok(None)` になるまで next_frame() を繰り返し、
        // 1 回の decode で生成されるすべての PCM を取得する
        while let Some(frame) = decoder
            .next_frame()
            .unwrap_or_else(|e| panic!("パケット {i} の next_frame でエラーが発生した: {e}"))
        {
            let frames = frame.len() / TEST_CHANNELS as usize;
            // 上限を 2 倍にしているのは、プライミングサンプル等で 1 パケットあたりの
            // 理論フレーム数をわずかに超えることがあるためである。
            // 同じパケットが複数回提供されるバグの場合は、この数倍のフレーム数が返る。
            assert!(
                frames <= AAC_FRAMES_PER_PACKET * 2,
                "1 回の next_frame が返したフレーム数が多すぎる: {frames} > {}",
                AAC_FRAMES_PER_PACKET * 2
            );
            total_frames += frames;
        }
    }

    // プライミングサンプル等を考慮し、総返却フレーム数は投入パケット数 × 1024 の近傍であることを確認する
    let expected = packets.len() * AAC_FRAMES_PER_PACKET;
    let tolerance = AAC_FRAMES_PER_PACKET * 2;
    assert!(
        total_frames >= expected.saturating_sub(tolerance) && total_frames <= expected + tolerance,
        "デコード済み総フレーム数 {total_frames} が期待範囲 [{}, {}] を超えた",
        expected.saturating_sub(tolerance),
        expected + tolerance
    );
}

/// 空のパケットを decode して finish した後、next_frame() を複数回呼んでもエラーにならない。
#[test]
fn finish_after_empty_decode_does_not_error() {
    let mut d = decoder_aac_stereo_48k();
    d.decode(&[]).expect("空の decode");
    d.finish().expect("finish 呼び出し");
    assert!(d.next_frame().expect("1 回目の next_frame").is_none());
    assert!(d.next_frame().expect("2 回目の next_frame").is_none());
}

/// 1 回の decode 後に next_frame() をループして、生成されたすべての PCM を取得できる。
#[test]
fn decode_then_loop_next_frame_consumes_all_output() {
    let packets = encode_aac_packets(1);
    assert!(
        !packets.is_empty(),
        "エンコーダーは 1 パケット以上生成しなければならない"
    );

    let mut decoder = decoder_aac_stereo_48k();
    decoder
        .decode(&packets[0])
        .expect("最初のパケットの decode");

    let mut total_frames = 0usize;
    while let Some(frame) = decoder.next_frame().expect("next_frame 呼び出し") {
        let frames = frame.len() / TEST_CHANNELS as usize;
        assert!(
            frames <= AAC_FRAMES_PER_PACKET * 2,
            "1 回の next_frame が返したフレーム数が多すぎる: {frames} > {}",
            AAC_FRAMES_PER_PACKET * 2
        );
        total_frames += frames;
    }

    assert!(
        total_frames > 0,
        "1 パケットのデコード結果から PCM が取得できなければならない"
    );
}

/// デコードエラー後も入力バッファがクリアされ、次のパケットを受け付けられる。
#[test]
fn decode_error_clears_input_buffer_and_recovers() {
    // 復帰用に正常な AAC-LC パケットを用意する。
    // 2 パケット以上あることを確認し、復帰時に使う正常パケットを確保する。
    let packets = encode_aac_packets(3);
    assert!(
        packets.len() >= 2,
        "エンコーダーは 2 パケット以上生成しなければならない"
    );

    // 壊れたパケットとして巨大な不正バイト列を使う。
    //
    // 正常パケットのペイロード上書きや切り詰めでは macOS 実機で
    // AudioConverterFillComplexBuffer がエラーを返さず Ok(None) になることを確認した。
    // 4096 バイトの不正データは実エラー (status=-50) を確実に誘発するためこちらを使う。
    let broken = vec![0xFFu8; 4096];

    let mut decoder = decoder_aac_stereo_48k();
    decoder
        .decode(&broken)
        .expect("壊れたパケットの decode 自体は成功しなければならない");
    let err = decoder
        .next_frame()
        .expect_err("壊れたパケットの next_frame はエラーを返さなければならない");
    assert!(
        err.to_string().contains("AudioConverterFillComplexBuffer"),
        "想定外のエラー: {err}"
    );

    // エラー後も previous packet not consumed で弾かれずに受理できる。
    decoder
        .decode(&packets[0])
        .expect("エラー直後の正常パケットは受理されなければならない");

    // 復帰後に PCM が取れることを確認する。
    let mut total_frames = 0usize;
    while let Some(frame) = decoder.next_frame().expect("復帰後の next_frame 呼び出し") {
        // デコーダー出力はステレオ固定のためチャンネル数 2 で割ってフレーム数を求める。
        total_frames += frame.len() / TEST_CHANNELS as usize;
    }
    assert!(
        total_frames > 0,
        "復帰後のデコード結果から PCM が取得できなければならない"
    );
}
