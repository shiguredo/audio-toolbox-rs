# `supported_codecs()` の照会対象を `EncoderCodec` / `DecoderCodec` と一致させる

Created: 2026-04-01
Completed: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`supported_codecs()` は本クレートの公開 API であるが、現状は `AudioCodecType::all()` に基づき **HE-AAC 系・FLAC・ALAC 等も含めて** AudioToolbox を照会する。一方、`Encoder` / `Decoder` が実際に選べるのは **`EncoderCodec` / `DecoderCodec` の部分集合**に限られる。

利用者が「このライブラリから見た対応コーデック一覧」を `supported_codecs()` で得たい場合、**一覧は実装の表面積と一致しているべき**である。未実装の種別まで照会結果に含めると、「OS にはあるがこのクレートでは選べない」というずれが残り、誤解を招く。

よって **`supported_codecs` は「今の実装が扱う種別だけ」を返す**形にし、照会対象を **フィルターした集合**とするのがよい（別 API で OS 全体を晒す必要は、本件のスコープではない）。

## 現状（事実）

※以下は **issue 起票時点** の記録。実装後は `codec_types_for_supported_codecs` を参照すること。

- `supported_codecs()` は `AudioCodecType::all()` の **9 種**を順に `probe_decoding` / `probe_encoding` する。
- `EncoderCodec` は **AAC-LC のみ**。`DecoderCodec` は **AAC-LC / MP3 / Opus**。
- 上記 9 種と `EncoderCodec` / `DecoderCodec` の対応は **一致しない**（例: FLAC / ALAC は照会されるが `DecoderCodec` に無い）。

## 目指す仕様

- `supported_codecs()` が返す `Vec<AudioCodecInfo>` の **`codec` の集合**は、**少なくとも `DecoderCodec` で表現できる種別をすべて含み**、**`EncoderCodec` で表現できる種別についてはエンコード情報も一貫して返す**。
- 具体的には、照会対象の `AudioCodecType` を **`AacLc` / `Mp3` / `Opus`** に限定する（順序は既存の `all()` と整合しやすい順でよい）。
- **HE-AAC 系・FLAC・ALAC 等は本 issue のスコープでは照会しない**（将来このクレートで `Decoder` 等に追加した時点で、照会対象の拡張を検討する）。

## `AudioCodecType` 列挙子について（本 issue でやらないこと）

- **`AudioCodecType` のバリアント（HE-AAC 系・FLAC・ALAC 等）は削除しない。** 本 issue が変えるのは **`supported_codecs()` が `probe_*` する対象の集合**だけである。
- 列挙子を残す理由の例: **後方互換**、将来の照会拡張や別 API での利用余地、`format_id` 等の既存 `match` との整合。
- **enum の縮小**（不要なバリアントの削除）は **公開 API の破壊的変更**になりうるため、必要なら **別 issue** で検討する（本 issue の完了条件に含めない）。

## 実装方針（案）

- `AudioCodecType` 内部の一覧（現 `all()`）を、**「照会対象」専用のスライス**に差し替えるか、名前を `types_aligned_with_encoder_and_decoder()` 等にし、**`supported_codecs` からだけ参照**する（**enum 定義自体は触らない**）。
- **照会前にフィルターする**（9 種すべてを probe してから捨てるのではなく、対象種別だけ `probe_*` する）。
- **公開ドキュメント**（`supported_codecs` の rustdoc、必要なら README）を、「**本クレートの `Encoder` / `Decoder` が扱うコーデックについて** OS 上の可否を返す」旨に更新する。
- **テスト**は、返却件数・内容が上記集合と一致することを前提に更新する（例: 件数 **3**、ALAC 前提の検証は削除）。`encoding.supported` は OS 依存になりうるため、**厳密に固定しすぎない**アサーションに整理する（既存の AAC-LC / MP3 のパターンに合わせる）。

## 完了条件

- [x] `supported_codecs()` の照会対象が `AacLc` / `Mp3` / `Opus` のみになっている。
- [x] rustdoc（および必要なら README）が仕様と一致している。
- [x] 関連テストが更新され、`cargo test` が通る。
- [x] `CHANGES.md` の `develop` に反映されている（未リリースのため **[ADD] に照会範囲を統合**し、別立ての [CHANGE] は不要とした）。

## スコープ外

- **`AudioCodecType` のバリアント削除**や **enum の縮小**（上記「列挙子について」参照）。
- **FLAC 等を `Decoder` で新規サポートする**実装（ExtAudioFile 等が要る可能性があるもの）は含めない。必要なら **別 issue** とする。
