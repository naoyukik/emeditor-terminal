---
name: architecture-validator
description: Validates the codebase against AcePilot's "Strict Rigid Layered Architecture" rules. Use after implementation tasks or before reviews to ensure physical isolation and dependency direction.
---

# Architecture Validator

This skill enforces the strict architectural boundaries defined in `conductor/code_styleguides/architecture_rules.md`.

## Core Validation Workflow

When this skill is active, perform the following automated and manual checks:

### 1. Win32 Pollution Check (Rule 2)
Windows API types and calls MUST be isolated in `_gui_driver.rs` and `_io_driver.rs`.
- **Search Command**: `grep_search` for `windows::Win32` or `use windows::` in `src/domain/` or `src/application/`.
- **Allowed Exception**: `src/gui/driver/` and `src/infra/driver/`.
- **Violation**: Any direct Win32 usage in Domain/Application layers is a **Critical** violation.

### 2. Protocol Leakage Check (Rule 3)
Protocol parsing (VTE, CSI, SGR) MUST be isolated in `_protocol_handler.rs`.
- **Search Command**: `grep_search` for `vte::Perform` or large `match` statements on CSI actions in `_entity.rs` files.
- **Violation**: `_entity.rs` should only handle pure buffer operations. Parsing context in Entity is a **High** violation.

### 3. Suffix Rule Compliance (Rule 1)
Verify that new or modified files follow the naming conventions:
- `_resolver.rs`: Presentation logic.
- `_gui_driver.rs`: Win32 UI/IME/GDI operations.
- `_workflow.rs`: Use-case orchestration.
- `_entity.rs` / `_value.rs`: Pure domain models.
- `_io_driver.rs`: Win32 I/O/ConPTY operations.

### 4. Safety Documentation (Rust Win32 Rule 3.1)
Every `unsafe` block MUST be preceded by a `// SAFETY:` comment.
- **Action**: Scan for `unsafe {` and ensure the comment exists and explains why the operation is safe.

## Reporting Format

Output a "Violation Report" structured as follows:

### [Severity] Rule Name - File:Line
- **Context**: Describe why this violates the architecture.
- **Required Action**: Concrete step to fix the isolation.
