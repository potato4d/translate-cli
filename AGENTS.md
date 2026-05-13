# AGENTS.md

このリポジトリでは、エージェントは次の開発スタイルで作業してください。

## 基本方針

- `docs/APPLICATION_DESIGN.md` を主要な仕様ソースとして扱う。
- 仕様に曖昧さや章ごとの揺れがある場合は、MVP スコープを優先し、実装判断を `docs/architecture/` に記録する。
- 作業内容、検証結果、残した判断は `docs/work-log/` に逐次記録する。
- 大きめの作業は「実装」「検証」「監査」のまとまりごとに区切る。

## 実装の進め方

- まず現状確認を行う。
  - `git status --short --branch`
  - 関連 docs の確認
  - 既存ファイル構成の確認
- 仕様から受け入れ条件をチェックリスト化してから実装する。
- Node.js / TypeScript 実装では標準ライブラリを優先し、依存追加は必要性が明確な場合だけにする。
- GitHub Release / Homebrew 向けの配布物は Bun の `--compile` で作る単体バイナリを前提にする。
- CLI、設定、Adapter、Prompt、Output Normalizer、Wizard などの責務を分離する。
- 実 Agent CLI はテストで直接呼ばず、`testdata/` の fake CLI で検証する。

## ドキュメント運用

- 実装・検証ログは `docs/work-log/YYYY-MM-DD-*.md` に書く。
- アーキテクチャ判断や責務分離は `docs/architecture/YYYY-MM-DD-*.md` に書く。
- 完了時は、仕様項目と実装・テスト・コマンド結果を対応づけた completion audit を `docs/work-log/` に残す。

## 検証

変更後は可能な限り次を実行する。

```sh
npm ci --prefix npm
npm test --prefix npm
node npm/dist/t.js --version
npm run test:binary --prefix npm
npm run build:release --prefix npm
git diff --check
```

`npm run build:release --prefix npm` は `dist/` に release archive、checksum、Homebrew Formula を生成する。Formula 配布が設計要件であるため、生成された Formula に Node.js 依存が入っていないことを確認する。

## Git 運用

- 他のエージェントやユーザーの変更が混在する可能性があるため、自分が編集したファイルだけを stage / commit する。
- 未関係の未追跡ファイルや変更は触らない。
- `git add .` は避け、対象ファイル・対象ディレクトリを明示して stage する。
- まとまった作業が完了したら commit し、push する。
- commit 前に staged file list を確認する。

```sh
git status --short --branch
git diff --cached --name-only
git commit -m "<message>"
git push
```

## 完了判定

完了前に必ず audit を行う。

- 目的を具体的な deliverables に言い換える。
- 仕様の各要求を、実ファイル・テスト・コマンド結果・push 状態に対応づける。
- proxy signal だけで完了扱いにしない。
  - テスト成功だけでなく、テストが要求をカバーしていることを確認する。
  - release 設定だけでなく、snapshot release で成果物名まで確認する。
- 不確実な項目が残る場合は完了扱いにせず、追加実装または追加検証を行う。

## サブエージェント活用

- 仕様の洗い出し、受け入れ条件の抽出、独立した調査はサブエージェントに任せてよい。
- 実装のクリティカルパスはローカルで進め、サブエージェントの結果は監査や抜け漏れ確認に使う。
