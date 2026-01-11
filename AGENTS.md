# Conventional Commits
日本語で記載してください。
Avoid overly verbose descriptions or unnecessary details.
Start with a short sentence in imperative form, no more than 50 characters long.

Add a prefix based on the following Type to the short sentence and separate them with a colon.
ex.) feat: add a function

## Type
Must be one of the following:

build: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
ci: Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)
docs: Documentation only changes
feat: A new feature
fix: A bug fix
perf: A code change that improves performance
refactor: A code change that neither fixes a bug nor adds a feature
style: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
test: Adding missing tests or correcting existing tests

## Reference URL
You can refer to the following URL when you need
- Conventional Commits: https://www.conventionalcommits.org/en/

## refの追加
1行目を記載したら2行目を空白行（改行のみ）にして、3行目に `ref: ` を記載し現在のブランチ名から `チケット番号` を記載してください。
チケット番号とは 現在のブランチ名の`^[0-9]+` の部分です。
例）
ブランチ名: 1234-content
ref: 1234
