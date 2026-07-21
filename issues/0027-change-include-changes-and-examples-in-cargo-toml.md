# Cargo.toml の include に CHANGES.md と examples/ を追加する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/change-cargo-include-changes-and-examples
- Polished:

## 目的

crates.io に配布されるアーカイブに `CHANGES.md` と `examples/sine_to_mp4.rs` が含まれるように `Cargo.toml` の `include` を修正する。現状は配布物からこれらが欠落しており、README / SKILL.md で紹介しているサンプル実行手順や変更履歴が crates.io 経由の利用者では確認できない。

## 優先度根拠

Medium とする。動作影響はないが、次回リリース (2026.2.0) 前に必ず整えたい。特に `CHANGES.md` は shiguredo-changelog 運用の前提であり、crates.io で欠落していると利用者が版差を追えない。

## 現状

`Cargo.toml:12`:

```toml
include = ["/LICENSE", "/README.md", "/build.rs", "/src/**"]
```

- `CHANGES.md` が含まれていない → crates.io 版で変更履歴が閲覧できない。
- `examples/sine_to_mp4.rs` が含まれていない → `cargo run --example sine_to_mp4` を README / SKILL.md で紹介しているのに crates.io 経由の利用者は使えない (ソースツリーからのみ使える)。

## 完了条件

- `Cargo.toml` の `include` に `/CHANGES.md` と `/examples/**` が追加される。
- `cargo package --list` で配布物に上記が含まれることを確認する。
- 意図的に除外している項目 (例: `/tests`) はコメントで残す判断でも可。

## 解決方法

`Cargo.toml` の `include` を以下に修正する。

```toml
include = ["/LICENSE", "/README.md", "/CHANGES.md", "/build.rs", "/src/**", "/examples/**"]
```

修正後 `cargo package --list` を実行し、`.crate` に含まれるファイル一覧を目視確認する。tests は依然として除外される (dev-dependencies の `shiguredo_mp4` を crates.io で入れる側に強制しないため)。
