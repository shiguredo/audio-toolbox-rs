# Makefile から実在しない pbt / fuzz 系ターゲットと .PHONY 誤字を削除する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-remove-non-existent-makefile-targets
- Polished: 2026-07-31

## 目的

Makefile に定義されているものの、参照先のパッケージやディレクトリが存在せず必ず失敗する `pbt` / `pbt-with-cover` / `fuzzing` / `fuzzing-list` ターゲットを削除する。合わせて `.PHONY` に列挙されている実在しないターゲット名 (`pbt-cover`, `fuzz`) を整理する。

## 優先度根拠

Medium とする。開発者が `make pbt` / `make fuzzing` を叩くと必ずエラーになる状態は AGENTS.md の「Don't live with broken windows」に反する。

## 現状

`Makefile` の `pbt` / `pbt-with-cover` / `fuzzing` / `fuzzing-list` ターゲットに以下の定義がある。

```makefile
# PBT を実行する
pbt:
	cargo test -p pbt

# PBT をカバレッジ付きで実行する
pbt-with-cover:
	cargo llvm-cov -p pbt --tests

# Fuzzing を全ターゲットで 30 秒ずつ実行する
fuzzing:
	@for target in $$(cargo fuzz list); do \
		echo "=== Fuzzing $$target ==="; \
		cargo +nightly fuzz run $$target -- -max_total_time=30 || exit 1; \
	done

# Fuzzing ターゲット一覧を表示する
fuzzing-list:
	cargo fuzz list
```

しかし:

- Cargo.toml は単一クレート構成 (`[workspace]` セクション無し) で `pbt` パッケージは存在しない。`cargo test -p pbt` は「package ID specification `pbt` did not match any packages」で失敗する。`pbt-with-cover` (`cargo llvm-cov -p pbt --tests`) も同じ `-p pbt` 指定で失敗する。
- リポジトリに `fuzz/` ディレクトリが実在しない。`cargo fuzz list` は失敗する。

加えて `.PHONY` 宣言に列挙されている `pbt-cover` は Makefile 内に存在しない (実体は `pbt-with-cover`)、`fuzz` も存在しない (実体は `fuzzing`)。実在ターゲットのうち `pbt-with-cover` は `.PHONY` に列挙されておらず、将来同名ファイルが混入した際にレシピが実行されない事故のリスクがある。`pbt-cover` / `fuzz` は存在しない名前の列挙で、実体名との不整合を残している。

## 完了条件

- `Makefile` から `pbt` / `pbt-with-cover` / `fuzzing` / `fuzzing-list` ターゲット (対応するコメント行を含む) が削除され、`grep -nEi "pbt|fuzz" Makefile` が 0 件になる。
- `.PHONY` 宣言の項目と実際のターゲット定義が一致する (`grep '^\.PHONY' Makefile` が `.PHONY: test cover check clippy fmt clean` になる)。
- `check` / `clippy` / `fmt` / `test` / `cover` / `clean` の実働ターゲットは影響を受けない。`make test` / `make check` / `make clippy` / `make fmt` が成功し、`cover` / `clean` は削除対象外のため構造上影響を受けない。
- `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`Makefile` から以下を削除する:

- `pbt:` ターゲット (`cargo test -p pbt`) とそのコメント行
- `pbt-with-cover:` ターゲット (`cargo llvm-cov -p pbt --tests`) とそのコメント行
- `fuzzing:` ターゲット (for ループ) とそのコメント行
- `fuzzing-list:` ターゲット (`cargo fuzz list`) とそのコメント行
- `.PHONY:` 宣言から `pbt`, `pbt-cover`, `fuzz`, `fuzzing`, `fuzzing-list` を削除

削除後の `.PHONY` は `test cover check clippy fmt clean` になる。

issue 0024 と同時期に対応する (変更対象が独立しているため順序は問わない)。issue 0021 も CHANGES.md の develop セクションに追記するため、マージ時にコンフリクトした場合は develop の最新を取り込んで解決する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: [UPDATE] Makefile から実在しない pbt / fuzz ターゲットを削除する)。

将来 Fuzz を導入する時点で `fuzz/` を用意して Makefile に追記し、PBT を導入する時点では dev-dependencies に proptest を追加してテスト内で利用する運用にする。導入計画があるなら別 issue として機能追加のカテゴリの issue を立てる。
