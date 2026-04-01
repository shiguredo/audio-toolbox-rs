//! `Decoder` の単体テスト（`src/lib.rs` のデコーダー部分に対応）
//!
//! PBT は PCM 変換を行わない（2 回目の `decode` は即 `Err`）。負荷の主因は
//! ケースごとの `AudioConverterNew` / `AudioConverterDispose` なので、既定 256 回は
//! 抑え、ケース数を明示する。

use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;

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

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        ..ProptestConfig::default()
    })]

    /// 最初の入力が 1 バイト以上ある場合、`next_frame` なしに 2 回目の `decode` は必ず失敗する。
    #[test]
    fn proptest_second_decode_fails_without_next_frame(
        first in prop::collection::vec(any::<u8>(), 1..512),
        second in prop::collection::vec(any::<u8>(), 1..512),
    ) {
        let mut d = decoder_aac_stereo_48k();
        d.decode(&first).expect("first decode");
        prop_assert!(d.decode(&second).is_err());
    }
}
