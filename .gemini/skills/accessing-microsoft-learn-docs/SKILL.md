---
name: accessing-microsoft-learn-docs
description: Searches and retrieves Microsoft Learn documentation via MCP Server. Provides access to Windows API, Win32, and related technology references. Use when looking up Windows API functions, system calls, or Microsoft platform documentation.
---

# Microsoft Learn ドキュメント検索

## 概要

`learn microsoft` MCP Serverを使用して、Microsoft Learnの公式ドキュメントを検索・取得する。

## 主な用途

- Windows API関数のリファレンス検索
- Win32 APIの使用方法確認
- システムコールやデータ構造の仕様確認
- COM/OLEインターフェースのドキュメント参照

## 検索のベストプラクティス

1. **具体的なAPI名で検索**: `CreateWindowExW`, `SendMessage`, `WM_PAINT` など
2. **機能カテゴリで検索**: `Console API`, `GDI functions`, `Window Messages` など
3. **エラーコードの調査**: `HRESULT`, `GetLastError` の戻り値を検索

## 関連リソース

- EmEditor Plugin SDK: プロジェクト内 `sdk/` ディレクトリ
- Windows API公式: https://learn.microsoft.com/windows/win32/
