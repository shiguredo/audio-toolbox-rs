# Cargo.toml の include に CHANGES.md と examples/ を追加する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/update-cargo-include-changes-and-examples
- Polished: 2026-09-05

## 目的

crates.io に配布されるアーカイブに `CHANGES.md` と `examples/` 配下のファイル全体が含まれるように `Cargo.toml` の `include` を修正する。現状は配布物からこれらが欠落しており、README / `skills/shiguredo-audio-toolbox/SKILL.md` で紹介しているサンプル (`examples/sine_to_mp4.rs`) が配布物に含まれないため crates.io のアーカイブからは参照・実行できず、変更履歴も確認できない。

## 優先度根拠

Medium とする。動作影響はないが、次回リリース (2026.2.0) 前に必ず整えたい。特に `CHANGES.md` は shiguredo-changelog 運用の前提であり、crates.io で欠落していると利用者が版差を追えない。

## 現状

`Cargo.toml` の `include` キー:

```toml
include = ["/LICENSE", "/README.md", "/build.rs", "/src/**"]
```

- `CHANGES.md` が含まれていない → crates.io 版で変更履歴が閲覧できない。
- `examples/sine_to_mp4.rs` が含まれていない → `cargo run --example sine_to_mp4` を README / `skills/shiguredo-audio-toolbox/SKILL.md` で紹介しているのに crates.io 経由の利用者は実行できない (ソースツリーからのみ)。

## 完了条件

1. `Cargo.toml` の `include` に `/CHANGES.md` と `/examples/**` が追加される (`grep -n '^include = ' Cargo.toml` が 1 行で、`/CHANGES.md` と `/examples/**` を含む)。
2. `cargo check --example sine_to_mp4` が成功し、`cargo run --example sine_to_mp4 -- --duration 1 --output /tmp/tone.mp4` で MP4 ファイルが生成される (配布物に含める example がコンパイル可能かつ実動すること。macOS 実機 (Xcode SDK あり) で確認する)。
3. `cargo package --list` の出力に `CHANGES.md` と `examples/sine_to_mp4.rs` が含まれ、`tests/` が出力に 1 件も現れないことを確認する。併せて `cargo package` が成功する (`.crate` が生成できること)。
4. `/tests` を意図的に除外する旨を `Cargo.toml` の `include` 行の上に日本語コメントで残す (`grep -n "tests" Cargo.toml` が 1 件で、`include` 行より前の行にある)。canary リリース後もコメントが残っていることを確認し、消えていた場合は再追加する。
5. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`Cargo.toml` の `include` を以下に修正する。

```toml
include = ["/LICENSE", "/README.md", "/CHANGES.md", "/build.rs", "/src/**", "/examples/**"]
```

`include` 行の上に「`/tests` は内部検証コードのため配布物から除外する」旨の日本語コメントを付ける。`tests` は `include` のパターンに含まれないため引き続き除外される。

修正後、macOS 実機で `cargo check --example sine_to_mp4` と `cargo run --example sine_to_mp4 -- --duration 1 --output /tmp/tone.mp4` が成功すること、`cargo package --list` の出力に `CHANGES.md` と `examples/sine_to_mp4.rs` が含まれ `tests/` が 1 件も現れないこと、`cargo package` が成功することを確認する。

本 issue は issue 0022 の完了後 (または同一リリース内) に実装する (0022 は CHANGES.md から issue 参照を除去する issue で、0027 の配布物変更より先に完了させる旨を明記している)。issue 0026 (docs.rs metadata 追加) と同じ Cargo.toml を編集するため、並行作業時はコンフリクトに注意する。issue 0021 等も CHANGES.md の develop / `### misc` に追記するため、マージ時にコンフリクトした場合は develop の最新を取り込んで解決する。issue 0025 (canary.py の tomli-w 化) は Cargo.toml のコメントを保持しないため、canary リリース後に本 issue で追加したコメントが残っていることを確認する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: [UPDATE] crates.io 配布物に CHANGES.md と examples/ を含める)。
