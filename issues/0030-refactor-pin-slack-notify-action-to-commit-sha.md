# slack-notify Action を commit SHA で固定する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-pin-slack-notify-action
- Polished: 2026-09-05

## 目的

`.github/workflows/ci.yml` と `.github/workflows/release.yml` が参照している `shiguredo/github-actions/.github/actions/slack-notify@main` を、他 action と同様に commit SHA で固定する。

## 優先度根拠

Medium とする。同組織の action ではあるが、`@main` 参照は上流の main ブランチ書き換えで CI / release ワークフローの再現性が失われる。d4cc68e は他組織 action (`actions/checkout` / `crates-io-auth-action`) のみを commit SHA で固定し、自組織 action は対象外としていたが (update-actions スキルも自組織 action のブランチ追従を「意図的」と分類している)、本 issue では再現性の観点から slack-notify も固定する。

## 現状

`.github/workflows/ci.yml` の `slack_notify` ジョブの Slack Notification ステップ:

```yaml
uses: shiguredo/github-actions/.github/actions/slack-notify@main
```

`.github/workflows/release.yml` の `slack_notify` ジョブの Slack Notification ステップ:

```yaml
uses: shiguredo/github-actions/.github/actions/slack-notify@main
```

一方 `actions/checkout` は以下のように SHA + バージョンコメントで固定されている。

```yaml
uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
```

なお `shiguredo/github-actions` リポジトリには現時点 (2026-07-31 確認) でタグが存在しない (`gh api repos/shiguredo/github-actions/tags` が空)。このため `actions/checkout` のような `# vX.Y.Z` 形式のバージョンコメントは付けられず、ブランチ名をコメントにする。

## 完了条件

1. `ci.yml` / `release.yml` の slack-notify 参照が `@<解決方法 1 で取得した 40 桁 SHA> # main` 形式に変わる (`grep -rn "slack-notify" .github/workflows/` で `@main` が 0 件、`grep -rnE "slack-notify@[0-9a-f]{40} # main" .github/workflows/` が 2 件)。取得した SHA が本 issue の記載と異なる場合は、取得した SHA で固定し、本 issue の記載も更新する。
2. 書き換え後に `ci.yml` の `slack_notify` ジョブが成功することを確認する (次回の CI 実行で)。release.yml は同一 SHA のため ci.yml の確認で代用する。
3. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

1. `shiguredo/github-actions` リポジトリの main ブランチ HEAD の commit SHA を取得する (`gh api repos/shiguredo/github-actions/commits/main --jq .sha`。2026-07-31 時点: `091812b41cd7c5f2b5818d4fde613f93bf0c93b1`)。
2. 両ワークフローを以下形式に書き換える。
   ```yaml
   uses: shiguredo/github-actions/.github/actions/slack-notify@091812b41cd7c5f2b5818d4fde613f93bf0c93b1 # main
   ```
3. `shiguredo/github-actions` に将来タグが作成されたら、`# main` を `# vX.Y.Z` に置き換えて更新する。

update-actions スキルはブランチ参照 (`@main`) を「対象外（報告のみ）」と分類し自動書き換えしないため、本 issue は手動で対応する。

issue 0029 と同じ release.yml / CHANGES.md を編集するため、並行作業時はコンフリクトに注意する (release.yml は別ジョブのため直接の衝突は少ないが、CHANGES.md の `### misc` への挿入位置は両 issue で同一のため衝突しうる)。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [UPDATE] slack-notify Action を commit SHA で固定する`)。
