# Iteration 12 Task List

**Version:** v12
**Date:** 2026-04-18
**Based on:** Spec v1.2 & Gap Analysis

---

## 1. Task Summary

| Category | Existing | New | Total | Status |
|----------|----------|-----|-------|--------|
| CLI | 6 | 0 | 6 | ✅ |
| API | 4 | 6 | 10 | ⬜ |
| Permissions | 4 | 0 | 4 | ✅ |
| Session | 4 | 0 | 4 | ✅ |
| Workspace | 5 | 0 | 5 | ✅ |
| Recovery | 0 | 3 | 3 | ⬜ |
| Web | 0 | 0 | 0 | ⬜ (P2) |
| **Total** | **23** | **9** | **32** | |

---

## 2. P1 Tasks (Iteration 12 Focus)

### 2.1 CLI Report Command

| Task ID | H-001 |
|---------|-------|
| Title | Implement CLI Report command |
| Description | Replace "not yet implemented" with full report generation |
| Module | cli |
| Priority | P1 |
| Estimated Time | 2 days |
| Status | Not Started |

**Acceptance Criteria:**
- `harness report --output json` produces valid JSON
- `harness report --output junit` produces valid JUnit XML
- `harness report --output md` produces Markdown report
- Report includes all ParityReport fields
- Gate evaluation result included

---

### 2.2 API Contract Definition

| Task ID | H-002 |
|---------|-------|
| Title | Create API Contract |
| Description | Define API contract YAML covering server lifecycle, /doc, auth, CORS, session/project interfaces, event subscription |
| Module | contracts/api |
| Priority | P1 |
| Estimated Time | 1 day |
| Status | ✅ Done |

**Acceptance Criteria:**
- `contracts/api/api_contract.yaml` exists
- Covers server startup/shutdown
- Covers /doc endpoint
- Covers authentication flow
- Covers CORS headers
- Covers session management endpoints
- Covers project management endpoints
- Covers message/command interface
- Covers file operations interface
- Covers tool registration interface
- Covers event subscription mechanism

---

### 2.3 API Tasks Expansion

| Task ID | H-003 |
|---------|-------|
| Title | Expand API Tasks to 8-10 |
| Description | Create 6 new API tasks to reach 10 total |
| Module | tasks/api |
| Priority | P1 |
| Estimated Time | 1-2 days |
| Status | ✅ Done |

**New Tasks to Create:**

| ID | Title | Entry Mode | Focus |
|----|-------|------------|-------|
| SMOKE-API-005 | Session create via API | API | Session creation |
| SMOKE-API-006 | Session resume via API | API | Session resumption |
| SMOKE-API-007 | Project list via API | API | Project enumeration |
| SMOKE-API-008 | Message send via API | API | Message sending |
| SMOKE-API-009 | Event subscribe via API | API | Event subscription |
| SMOKE-API-010 | Tool list via API | API | Tool enumeration |

---

### 2.4 Recovery Tasks Creation

| Task ID | H-004 |
|---------|-------|
| Title | Create Recovery Tasks |
| Description | Create 3 recovery tasks for session reconnect/interrupted operation recovery |
| Module | tasks/recovery |
| Priority | P1 |
| Estimated Time | 1 day |
| Status | ✅ Done |

**New Tasks to Create:**

| ID | Title | Entry Mode | Focus |
|----|-------|------------|-------|
| SMOKE-RECOVERY-001 | Session reconnect | Server | Session reconnection |
| SMOKE-RECOVERY-002 | Interrupted operation recovery | Server | Recovery after interruption |
| SMOKE-RECOVERY-003 | Server restart handling | Server | Behavior during server restart |

---

## 3. P2 Tasks (Post Iteration 12)

### 3.1 Event Contract Definition

| Task ID | H-005 |
|---------|-------|
| Title | Create Event Contract |
| Description | Define event stream contract covering event types, payloads, subscription lifecycle |
| Module | contracts/events |
| Priority | P2 |
| Estimated Time | 0.5 day |
| Status | Done |

---

### 3.2 Report Type Naming Cleanup

| Task ID | H-006 |
|---------|-------|
| Title | Unify Report type naming |
| Description | Rename types/report.rs::Report to SimpleReport to avoid confusion with ParityReport |
| Module | types |
| Priority | P2 |
| Estimated Time | 0.5 day |
| Status | ✅ Done |

---

### 3.3 Configuration Externalization

| Task ID | H-007 |
|---------|-------|
| Title | Externalize configuration |
| Description | Move hardcoded thresholds (pass_rate, timeouts) to config files |
| Module | config |
| Priority | P2 |
| Estimated Time | 1 day |
| Status | ✅ Done |

---

## 4. Existing Tasks Status

### 4.1 CLI Tasks (6 tasks - Complete)

| ID | Title | Status |
|----|-------|--------|
| SMOKE-CLI-001 | CLI help command displays usage | ✅ |
| SMOKE-CLI-002 | CLI version command displays version | ✅ |
| SMOKE-CLI-003 | CLI serve command starts server | ✅ |
| SMOKE-CLI-004 | CLI stats command shows stats | ✅ |
| SMOKE-CLI-005 | CLI session command manages sessions | ✅ |
| SMOKE-CLI-006 | CLI export command exports data | ✅ |

### 4.2 API Tasks (4 existing + 6 new = 10 total)

| ID | Title | Status |
|----|-------|--------|
| SMOKE-API-001 | API health endpoint returns status | ✅ |
| SMOKE-API-002 | API status endpoint returns server status | ✅ |
| SMOKE-API-003 | API version endpoint returns version info | ✅ |
| SMOKE-API-004 | API metrics endpoint returns server metrics | ✅ |
| SMOKE-API-005 | Session create via API | ⬜ New |
| SMOKE-API-006 | Session resume via API | ⬜ New |
| SMOKE-API-007 | Project list via API | ⬜ New |
| SMOKE-API-008 | Message send via API | ⬜ New |
| SMOKE-API-009 | Event subscribe via API | ⬜ New |
| SMOKE-API-010 | Tool list via API | ⬜ New |

### 4.3 Permissions Tasks (4 tasks - Complete)

| ID | Title | Status |
|----|-------|--------|
| SMOKE-PERM-001 | Build permission writes files | ✅ |
| SMOKE-PERM-002 | Plan permission reads without writing | ✅ |
| SMOKE-PERM-003 | Ask permission queries without changes | ✅ |
| SMOKE-PERM-004 | Permission denied for restricted operations | ✅ |

### 4.4 Session Tasks (4 tasks - Complete)

| ID | Title | Status |
|----|-------|--------|
| SMOKE-SESSION-001 | Session create | ✅ |
| SMOKE-SESSION-002 | Session switch | ✅ |
| SMOKE-SESSION-003 | Session terminate | ✅ |
| SMOKE-SESSION-004 | Session persist across restarts | ✅ |

### 4.5 Workspace Tasks (5 tasks - Complete)

| ID | Title | Status |
|----|-------|--------|
| SMOKE-WS-001 | Workspace create | ✅ |
| SMOKE-WS-002 | Workspace file operations | ✅ |
| SMOKE-WS-003 | Workspace git operations | ✅ |
| SMOKE-WS-004 | Workspace cleanup on exit | ✅ |
| SMOKE-WS-005 | Workspace config preservation | ✅ |

### 4.6 Recovery Tasks (0 existing + 3 new)

| ID | Title | Status |
|----|-------|--------|
| SMOKE-RECOVERY-001 | Session reconnect | ⬜ New |
| SMOKE-RECOVERY-002 | Interrupted operation recovery | ⬜ New |
| SMOKE-RECOVERY-003 | Server restart handling | ⬜ New |

### 4.7 Web Tasks (0 existing, 0 new - P2)

| ID | Title | Status |
|----|-------|--------|
| - | - | Pending P2 |

---

## 5. Contract Coverage

| Contract ID | Name | Status |
|-------------|------|--------|
| CLI-CONTRACT-001 | CLI Basic Execution Contract | ✅ |
| PERMISSION-CONTRACT-001 | Permission Contract | ✅ |
| WORKSPACE-SIDE-EFFECT-001 | Workspace Side Effect Contract | ✅ |
| STATE-MACHINE-001 | Session Contract | ✅ |
| API-CONTRACT-001 | Server/API Contract | ✅ Done |
| EVENT-CONTRACT-001 | Event Stream Contract | ⬜ New (P2) |

---

## 6. Task Dependencies

```
H-001 (CLI Report) ──────┬── H-002 (API Contract)
      │                  │
      │                  └── H-003 (API Tasks)
      │                              │
      │                              └── H-004 (Recovery Tasks)
      │
      └── H-005 (Event Contract) ── H-006 (Report Naming) ── H-007 (Config Externalization)
```

---

## 7. Verification Commands

| Command | Expected Result |
|---------|-----------------|
| `cargo test report_schema_smoke_tests` | All tests pass |
| `cargo test suite_selection_smoke_tests` | All tests pass |
| `cargo test ci_gate_smoke_tests` | All tests pass |
| `cargo run -- report --output json` | Valid JSON output |
| `cargo run -- report --output junit` | Valid JUnit XML output |
| `cargo run -- report --output md` | Valid Markdown output |

---

*Task List Version: v12*
*Created: 2026-04-18*
*Next Review: Iteration 13