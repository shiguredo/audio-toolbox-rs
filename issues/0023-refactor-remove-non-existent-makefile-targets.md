# Makefile から実在しない pbt / fuzzing ターゲットと .PHONY 誤字を削除する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-remove-non-existent-makefile-targets
- Polished:

## 目的

Makefile に定義されているものの、参照先のパッケージやディレクトリが存在せず必ず失敗する `pbt` / `pbt-with-cover` / `fuzzing` / `fuzzing-list` ターゲットを削除する。合わせて `.PHONY` に列挙されている実在しないターゲット名 (`pbt-cover`, `fuzz`) を整理する。

## 優先度根拠

Medium とする。開発者が `make pbt` / `make fuzzing` を叩くと必ずエラーになる状態は AGENTS.md「Don't live with broken windows」に反する。PBT / Fuzz を将来導入する予定があるならその時点で追加する運用に切り替える。

## 現状

`Makefile:12-28` に以下のターゲットが定義されている。

```makefile
pbt:
	cargo test -p pbt

pbt-with-cover:
	cargo llvm-cov -p pbt --tests

fuzzing:
	@for target in $$(cargo fuzz list); do \
		echo "=== Fuzzing $$target ==="; \
		cargo +nightly fuzz run $$target -- -max_total_time=30 || exit 1; \
	done

fuzzing-list:
	cargo fuzz list
```

しかし:

- Cargo.toml は単一クレート構成 (`[workspace]` セクション無し) で `pbt` パッケージは存在しない。`cargo test -p pbt` は「package `pbt` is not found」で失敗する。
- リポジトリに `fuzz/` ディレクトリが実在しない。`cargo fuzz list` は失敗する。

加えて `Makefile:1` の `.PHONY` に列挙されている `pbt-cover` は Makefile 内に存在せず (実体は `pbt-with-cover`)、`fuzz` も存在しない (実体は `fuzzing`)。名前の食い違いは将来同名ファイル混入時にターゲット扱いされない事故を招く。

## 完了条件

- 実在しないパッケージ / ディレクトリを参照するターゲットが Makefile から削除される。
- `.PHONY` 宣言と実際のターゲット名が完全に一致する。
- 削除に伴い `check`, `clippy`, `fmt`, `test`, `cover`, `clean` 等の実働ターゲットは影響を受けない。
- 併せて issue 0024 (test_decoder.rs 冒頭コメントの虚偽 fuzz 保証除去) と作業タイミングを合わせる。

## 解決方法

`Makefile` から以下を削除する:

- `pbt:` および `cargo test -p pbt` の行
- `pbt-with-cover:` および `cargo llvm-cov -p pbt --tests` の行
- `fuzzing:` および対応する for ループ
- `fuzzing-list:` および `cargo fuzz list` の行
- `.PHONY:` から `pbt`, `pbt-cover`, `fuzz`, `fuzzing`, `fuzzing-list` を削除

将来 PBT / Fuzz を導入する時点で `pbt/` / `fuzz/` を用意し、Makefile に追記する運用にする。導入計画があるなら別 issue として `feature/add-pbt-suite` などを立てる。
