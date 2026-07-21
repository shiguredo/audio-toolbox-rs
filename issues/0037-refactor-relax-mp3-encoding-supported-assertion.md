# supported_codecs_mp3_decode_only_typically の脆いアサーションを緩める

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-relax-mp3-encoding-supported-assertion
- Polished:

## 目的

`tests/test_codec_info.rs::supported_codecs_mp3_decode_only_typically` (`tests/test_codec_info.rs:33-41`) の `assert!(!mp3.encoding.supported)` が macOS の OS 実装依存 (MP3 エンコーダの有無) に張り付いており、将来 macOS が MP3 エンコード対応した瞬間にテストが赤くなる脆さを解消する。

## 優先度根拠

Medium とする。現時点では動作しているが、テスト名の `typically` 自称通り本来固定してはいけない事柄を assert しており、将来の macOS 更新で予告なく CI が落ちる。SKILL.md L325 では Opus について同じ懸念を「本クレートのテストでも encode 側は検証していない」として正しく回避しているため、MP3 だけ逆張りしているのは整合性がない。

## 現状

`tests/test_codec_info.rs:33-41`:

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

`assert!(!mp3.encoding.supported)` は AudioToolbox が「MP3 エンコーダーを返さない」という OS 実装事実を仕様として固定してしまう。同様の懸念について SKILL.md `## 既知の制限事項` L325 は Opus について「本クレートのテストでも encode 側は検証していない」と明記しているが、MP3 だけ逆張りしている。

## 完了条件

- `assert!(!mp3.encoding.supported)` が削除されるか、より緩い不変条件テスト (「エンコード非対応時に `bitrate_control_modes` が空」) に置き換わる。
- テスト名の `typically` も内容と整合する名前に変更する。
- 将来 macOS が MP3 エンコード対応してもテストが赤くならない。

## 解決方法

以下のいずれかで対応する。

### 案 A: assertion を削除

- `assert!(!mp3.encoding.supported)` を削除。
- テスト名を `supported_codecs_mp3_decode_supported` に変更。
- decode 側の assert (`assert!(mp3.decoding.supported)`) だけ残す。

### 案 B: 不変条件テストに置き換え

- `assert!(!mp3.encoding.supported || !mp3.encoding.bitrate_control_modes.is_empty())` (エンコード非対応なら bitrate_control_modes は空である、という不変条件)。
- あるいは「エンコード非対応時に `bitrate_control_modes` が必ず空」の判定を専用テストで書く。

案 A のほうがシンプル。実装コスト対効果を優先するなら案 A を推奨。

なお、issue 0018 で `src/lib.rs::tests::test_supported_codecs` を削除する際にも同じ脆さが含まれる。issue 0018 と対応順序を調整する。
