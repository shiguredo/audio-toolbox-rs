# `AudioStreamBasicDescription` / `AudioBufferList` の `zeroed().assume_init()` を安全な初期化に置き換える

Created: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

`MaybeUninit::<T>::zeroed().assume_init()` は、**型 `T` が全ビット 0 を合法状態として許す**場合に限り安全とされる。bindgen 生成型と **Apple ヘッダの定義**の差により、**未定義動作**（結果として **セグフォ** 等）のリスクがある。

`AudioBufferList` の `assume_init` は **`Encoder::new` / `Decoder::new` だけでなく**、`encode_impl` / `decode_impl` でも **呼び出しのたびに**行われている。置き換え時は **全箇所**を同方針で扱う。

## 受け入れ条件の目安

- `src/lib.rs` および `src/codec_info.rs` の該当箇所を、**未定義動作を起こさない初期化**に置き換える。
- 振る舞いは **現状と同等**（リグレッションテストで確認）。

## 参考（該当コード）

- `src/lib.rs`: `Encoder::new`、`Decoder::new`、`encode_impl`、`decode_impl`
- `src/codec_info.rs`: `create_probe_converter`
