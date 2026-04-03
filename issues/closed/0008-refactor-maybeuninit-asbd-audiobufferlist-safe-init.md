# `AudioStreamBasicDescription` / `AudioBufferList` の `zeroed().assume_init()` を安全な初期化に置き換える

Created: 2026-04-01
Completed: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

`MaybeUninit::<T>::zeroed().assume_init()` は、**型 `T` に対して全ビット 0 が合法な値か**が Rust の意味論上問題になる。ただし **少なくとも現状**、`AudioStreamBasicDescription` / `AudioBufferList` / `AudioBuffer` は **主に整数・ポインタ等のフィールド**で構成されており、**全ビット 0 を直ちに不正な値だと断定できる根拠は薄い**。「bindgen とヘッダの差で必ず UB」と **根拠なく強く言うと、真に致命的な unsafe 問題と同列に並び優先度を誤る**。

本 issue の位置づけは次の **いずれか（または併用）** とする。

1. **安全性・意図の明文化:** `assume_init` をやめ、**フィールド単位の初期化や `MaybeUninit` の適切なパターン**に置き換え、**どの不変条件で安全か**をコメントで固定する **リファクタリング候補**。
2. **UB として扱う場合:** **ゼロ初期化が `T` の合法値にならない**ことを、レイアウト・ドキュメント・Rust の validity に照らして **具体的に立証できる**ときに、未定義動作リスクとして優先度を上げる。

`AudioBufferList` の初期化は **`Encoder::new` / `Decoder::new` だけでなく**、`encode_impl` / `decode_impl` でも **呼び出しのたびに**行われている。置き換え時は **全箇所**を同方針で扱う。

## 受け入れ条件の目安

- `src/lib.rs` および `src/codec_info.rs` の該当箇所を、**`zeroed().assume_init()` に依存しない**初期化に置き換える（**意図と安全条件をコメントで明文化**する）。
- 振る舞いは **現状と同等**（リグレッションテストで確認）。
- **「未定義動作を確実に除去した」**と主張する場合は、**対象型 `T` について全ビット 0 が合法でない**根拠を issue またはコメントに **明示**する。

## ミッション適合性の確認

- **適合する（優先度は上記の立証の有無で調整）。** 根拠: `assume_init` は誤用しやすく、将来の型定義差やレビュー負荷の観点で **明文化された初期化**に寄せる価値がある。**断定 UB** と **保守上のリファクタ**を混同しないこと。
- **注意:** **ゼロが常に合法と静的に言い切れない**一方で、**即時 UB とまで言えない**なら **P1 固定ではなく「リファクタ候補」**として扱い、他の **実害の明確な** unsafe 修正と優先順位を比較する。

## 参考（該当コード）

- `src/lib.rs`: `Encoder::new`、`Decoder::new`、`encode_impl`、`decode_impl`
- `src/codec_info.rs`: `create_probe_converter`

## 解決方法

- `audio_stream_basic_description_zeroed` / `audio_buffer_list_placeholder` で **全フィールドを明示的にゼロ初期化**し、`MaybeUninit::zeroed().assume_init()` を廃止した（`src/lib.rs`、`src/codec_info.rs`）。
