# エンコーダー・コールバック内の整数乗算・キャストを checked にする

Created: 2026-04-01
Completed: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

コールバック内で `packets as usize * channels`、`packets * channels as u32 * size_of::<i16>()`、`(this.pcm_buf.len() / channels) as u32` 等を用いている。**乗算のオーバーフロー**や **`usize` から `u32` への切り詰め**により、**条件判定や `drain` 範囲が壊れ**、**パニック**（例: `drain` の範囲不正）や **不正なメモリ操作**につながりうる。

**補足（Decoder）:** `io_number_data_packets` の **null 検査**は `issues/0006-hardening-ffi-callback-null-and-m-data.md` と合わせて扱う。また `this.encoded_buf.len() as u32` により、**`len()` が `u32::MAX` を超える**と切り詰めで **報告サイズと実バッファ**が不整合になりうる（実用上は稀）。

## 受け入れ条件の目安

- オーバーフローしうる演算は **`checked_mul` / `try_from`** 等に置き換え、失敗時は **安全側**（エラーコード返却または `K_NO_MORE_INPUT` 等、設計に合わせる）。
- 32bit ターゲットでも **論理が破綻しない**こと。

## ミッション適合性の確認

- **適合する。** 根拠: 乗算オーバーフローや **不正な `drain` 範囲**は **`Vec::drain` でパニック**（境界外）や、**誤った長さのスライス**経由の **UB** につながりうる。
- **注意:** 通常の `io_number_data_packets` では発火しにくいが、**異常値・将来の変更**に対する防御としてミッションに含める。

## 参考（該当コード）

- `src/lib.rs`: `Encoder::callback`（`packets`、`channels`、`drain` 範囲）、`Decoder::callback`（`io_number_data_packets` の逆参照）

## 解決方法

- `Encoder::callback` で `checked_mul` / `u32::try_from` を用い、乗算オーバーフローや不正な `drain` 範囲を避けるようにした。
- `Decoder::callback` で `encoded_buf.len()` を `u32::try_from` し、切り詰めが起きる場合は `kAudio_ParamError` を返すようにした。
