# supported_codecs_mp3_decode_only_typically の脆いアサーションを緩める

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-relax-mp3-encoding-supported-assertion
- Polished: 2026-07-31

## 目的

`tests/test_codec_info.rs` の `supported_codecs_mp3_decode_only_typically` の `assert!(!mp3.encoding.supported)` が macOS の OS 実装依存 (MP3 エンコーダの有無) に張り付いており、将来 macOS が MP3 エンコード対応した瞬間にテストが赤くなる脆さを解消する。

## 優先度根拠

Medium とする。現時点では動作しているが、テスト名の `typically` 自称通り本来固定してはいけない事柄を assert しており、将来の macOS 更新で予告なく CI が落ちる。`skills/shiguredo-audio-toolbox/SKILL.md` の「既知の制限事項」では Opus について同じ懸念を「本クレートのテストでも encode 側は検証していない」として正しく回避しているため、MP3 だけ逆張りしているのは整合性がない。

## 現状

`tests/test_codec_info.rs` の `supported_codecs_mp3_decode_only_typically`:

```rust
#[test]
fn supported_codecs_mp3_decode_only_typically() {
    let codecs = supported_codecs();
    let mp3 = codecs
        .iter()
        .find(|c| c.codec == AudioCodecType::Mp3)
        .expect("MP3 entry");
    assert!(mp3.decoding.supported);
    assert!(!mp3.encoding.supported);
}
```

`assert!(!mp3.encoding.supported)` は AudioToolbox が「MP3 エンコーダーを返さない」という OS 実装事実を仕様として固定してしまう。同様の懸念について SKILL.md の「既知の制限事項」は Opus について「本クレートのテストでも encode 側は検証していない」と明記しているが、MP3 だけ逆張りしている。なお同種の脆いアサーションは `src/lib.rs` の `test_supported_codecs` (issue 0018 で削除予定) にも存在する。

## 完了条件

1. `assert!(!mp3.encoding.supported)` が削除され、decode 側の assert のみが残る (案 A で確定)。
2. テスト名が `supported_codecs_mp3_decoding_supported` に変更される (既存の `supported_codecs_opus_decoding_supported` と命名を揃える)。
3. テストが `mp3.encoding.supported` を参照しないこと (`tests/test_codec_info.rs` 内の `mp3.encoding.supported` 参照が 0 件であることを grep で確認。AAC-LC テストの `aac_lc.encoding.supported` は対象外のため残る)。
4. `cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` が成功する (`--all-targets` はテストターゲットも検証するため意図的に使用する。ci.yml の clippy はテストターゲットを含まない。`--test-threads=1` は issue 0038 の結論に従う)。
5. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない (issue 0022 の管轄)。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`assert!(!mp3.encoding.supported)` を削除し、テスト名を `supported_codecs_mp3_decoding_supported` に変更する (案 A で確定。案 B の不変条件テストは式の複雑さに対して効果が薄く、将来 macOS が MP3 エンコード対応した場合にビットレート制御モードが空になる経路 (converter 作成失敗) が残るため不採用)。

1. `tests/test_codec_info.rs` の `supported_codecs_mp3_decode_only_typically` の `assert!(!mp3.encoding.supported)` を削除する。
2. テスト名を `supported_codecs_mp3_decoding_supported` に変更する。
3. `tests/test_codec_info.rs` 内の `mp3.encoding.supported` 参照が 0 件であることを grep で確認する (完了条件 3 に対応)。
4. 最後に `cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` で確認する。
5. `CHANGES.md` の develop / `### misc` に [UPDATE] エントリを追記する (担当者行付き、issue 番号・issue ファイル名を含めない)。

スコープは `tests/test_codec_info.rs` のみ。`skills/shiguredo-audio-toolbox/SKILL.md` の「既知の制限事項」は本 issue では変更しない (Opus エンコード対応の OS 依存性の記述は優先度根拠の参考として引用しているのみで、MP3 の明記追加は対象外)。`tests/test_codec_info.rs` の AAC-LC テストの `bitrate_control_modes` 非空 assert も同種の OS 依存があるが、AAC-LC はエンコーダーが macOS の恒久機能であり本 issue の対象外とする。

issue 0018 を先に実施し、その後に着手する (0018 で `src/lib.rs` の `test_supported_codecs` が削除されるため、lib.rs 側の脆いアサーションは本 issue の対象外)。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [UPDATE] supported_codecs の MP3 エンコード非対応アサーションを緩める`)。
