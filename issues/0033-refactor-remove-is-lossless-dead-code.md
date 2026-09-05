# AudioCodecType::is_lossless() のデッドコードを削除する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-remove-is-lossless-dead-code
- Polished: 2026-09-05

## 目的

`AudioCodecType::is_lossless()` は現状の probe 対象 (AAC-LC / MP3 / Opus) では常に false になるデッドコードのため削除する。呼び出し箇所の分岐もリテラル化する。

## 優先度根拠

Medium とする。動作は正しいが、読者に「無効な分岐」と誤解させ、将来 FLAC / ALAC を追加する際にも本来的な意味を再検討する必要がある。「Don't live with broken windows」の観点から早めに整理する。

## 現状

`src/codec_info.rs` の `AudioCodecType::is_lossless` の定義:

```rust
fn is_lossless(self) -> bool {
    matches!(self, Self::Flac | Self::Alac)
}
```

`src/codec_info.rs` の `create_probe_converter` 内の唯一の呼び出し:

```rust
output_format.mBitsPerChannel = if codec.is_lossless() { 16 } else { 0 };
```

`AudioCodecType::codec_types_for_supported_codecs` が返すのは `AacLc`, `Mp3`, `Opus` の 3 種のみ。`create_probe_converter` はこの 3 種に対してしか呼ばれないので、`is_lossless()` は常に false を返す。

## 完了条件

1. `AudioCodecType::is_lossless` が削除される (`grep -rn "is_lossless" src/` が 0 件)。
2. `create_probe_converter` の分岐がリテラル `0` に置き換わり、コメントで「ロスレスコーデックではエンコード出力の `mBitsPerChannel` に入力のビット深度を設定する必要があるが、現状の probe 対象は非ロスレスコーデックのみのため 0 を設定する」の旨を残す。
3. probe 対象外バリアント (`AacHe` / `AacHeV2` / `AacLd` / `AacEld` / `Flac` / `Alac`。コード上は `format_id` 等の match で使用されるが probe では構築されない) の扱いを検討する別 issue が issues/ に作成される (将来 FLAC / ALAC を probe に含める際の設計 (`is_lossless` の再導入もしくは probe 対象ごとに `mBitsPerChannel` を指定) はその検討の一部として含める)。着手時に issues/ を確認し、該当する別 issue が存在しない場合は本 issue 側で作成する (別 issue は develop 上で create-issue 経由で作成する)。
4. 既存の全テストが引き続きパスし、`cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --workspace -- -D warnings` が成功する (`--test-threads=1` は issue 0038 の結論に従う)。
5. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

1. `src/codec_info.rs` の `AudioCodecType::is_lossless` を削除する。
2. `src/codec_info.rs` の `create_probe_converter` 内を `output_format.mBitsPerChannel = 0;` に置き換え、コメントで理由を明記する (「ロスレスコーデックではエンコード出力の `mBitsPerChannel` に入力のビット深度を設定する必要があるが、現状の probe 対象は非ロスレスコーデックのみのため 0 を設定する」。既存の「// ロスレスコーデックでは入力のビット深度を設定する」コメントは新しいコメントに置き換える)。ステップ 1・2 は同一コミットにする (中間状態でコンパイルエラーになるため)。
3. 最後に `cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --workspace -- -D warnings` で確認する (`--test-threads=1` は issue 0038 の結論に従う)。
4. probe 対象外バリアントの扱いと将来 FLAC / ALAC を probe に含める際の設計検討は、完了条件 3 の別 issue で行う (着手時に issues/ を確認し、既存の issue がなければ本 issue 側で立てる。別 issue は develop 上で create-issue 経由で作成する)。issue 0003 で「列挙子は削除しない」判断があるが、`is_lossless` 削除と合わせて再検討する余地がある。
5. `CHANGES.md` の develop / `### misc` に shiguredo-changelog スキルに従って [UPDATE] エントリを追記する (追記エントリに issue 番号・issue ファイル名は含めない)。

issue 0032 と同ファイル (`src/codec_info.rs`) を編集するため、マージ順・コンフリクト対応に注意する。`CHANGES.md` の develop / `### misc` に同時期に追記する他 issue とマージ時にコンフリクトした場合は、develop の最新を取り込んで解決する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [UPDATE] デッドコードの is_lossless() を削除する`)。
