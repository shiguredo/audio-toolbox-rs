# デコード経路で出力バッファと `mDataByteSize` を整合させセグフォ・パニックを防ぐ

Created: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

`Decoder::decode_impl` は `AudioConverterFillComplexBuffer` に **`pcm_buf` のポインタとバッファサイズ**を渡し、成功後に `mDataByteSize` を読む。

**セグフォ・UB:** フレームワークが **`mDataByteSize` や実際の書き込み量が、渡したバッファの範囲を超える**動作をすると、**バッファオーバーフロー**となり **セグフォまたは未定義動作**になる。

**パニック:** `Vec::truncate` は `len` が現在長より大きい場合 **何もしない**（パニックしない）。現行コードで **`size` が `pcm_buf.len()` を超える**と縮めずに返す経路がありうるが、本 issue の主眼は **FFI 越境**である。いずれにせよ **`mDataByteSize` を信じたまま後続でスライス等を誤るとパニック**しうる箇所があれば、検証で潰す。

## 既存 issue との関係

- `issues/0002-investigate-decoder-output-buffer-vs-codec-limits.md` は定数・理論上限の調査。本 issue は **実行時の `mDataByteSize` とバッファ境界**の検証。

## 受け入れ条件の目安

- 返却された `mDataByteSize`（および導出するサンプル数）が **確保した出力バッファを超えない**ことを検証し、異常時は **`Result::Err`**（パニックにしない）。
- `0002` の定数方針と矛盾しないこと。

## 参考（該当コード）

- `src/lib.rs`: `decode_impl`、`DECODE_BUF_FRAMES`、`OUTPUT_CHANNELS`
