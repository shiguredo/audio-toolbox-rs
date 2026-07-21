# slack-notify Action を commit SHA + バージョンコメントで固定する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-pin-slack-notify-action
- Polished:

## 目的

`.github/workflows/ci.yml` と `.github/workflows/release.yml` が参照している `shiguredo/github-actions/.github/actions/slack-notify@main` を、他 action と同様に commit SHA + バージョンコメントで固定する。

## 優先度根拠

Medium とする。同組織の action ではあるが、`@main` 参照は上流の main ブランチ書き換えで CI / release ワークフローの再現性が失われる。プロジェクトは既に `d4cc68e` で「GitHub Actions の外部 Action を commit SHA で固定する」を実施しており、方針からの逸脱を残すべきでない。

## 現状

`.github/workflows/ci.yml:62`:

```yaml
uses: shiguredo/github-actions/.github/actions/slack-notify@main
```

`.github/workflows/release.yml:62`:

```yaml
uses: shiguredo/github-actions/.github/actions/slack-notify@main
```

一方 `actions/checkout` は以下のように SHA + バージョンコメントで固定されている。

```yaml
uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
```

## 完了条件

- 両ワークフローの `slack-notify` 参照が `@<commit SHA> # <tag>` 形式に変わる。
- `/update-actions` スキルで再度スキャンして違反 0 件になる。

## 解決方法

1. `shiguredo/github-actions` リポジトリで `slack-notify` の最新タグを特定する (例: `v1.0.0`)。
2. 対応する commit SHA を `gh api repos/shiguredo/github-actions/git/refs/tags/<tag>` で取得する。
3. 両ワークフローを以下形式に書き換える。
   ```yaml
   uses: shiguredo/github-actions/.github/actions/slack-notify@<40 桁 SHA> # v1.0.0
   ```
4. 変更を CHANGES.md `### misc` に記載する。

`/update-actions` スキルを回すと自動的にこの作業ができる可能性がある。ただし本 issue では「対象を確実に指定できる」ことを重視して手動で行ってもよい。
