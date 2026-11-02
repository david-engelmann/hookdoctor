# hookdoctor examination

Format hookdoctor-report/1
Configuration source synthetic-settings.json
Session sources synthetic-session.jsonl
Window start None end None

## Hooks

### guard-bash.sh

Identifier PreToolUse:0:0
Event PreToolUse
Matcher Bash
Class firing
Matched 1
Ran 1
Effects denial 1, modification 0, consumed output 0
First ran at 2026-01-01T09:00:05Z
Last ran at 2026-01-01T09:00:05Z

### lint-edit.py

Identifier PreToolUse:1:0
Event PreToolUse
Matcher Edit|Write
Class firing-without-effect
Matched 1
Ran 1
Effects denial 0, modification 0, consumed output 0
First ran at 2026-01-01T09:02:00Z
Last ran at 2026-01-01T09:02:00Z

### audit-all.sh

Identifier PreToolUse:2:0
Event PreToolUse
Matcher 
Class firing
Matched 2
Ran 2
Effects denial 1, modification 0, consumed output 0
First ran at 2026-01-01T09:00:05Z
Last ran at 2026-01-01T09:02:00Z

### log-bash.sh

Identifier PostToolUse:0:0
Event PostToolUse
Matcher Bash
Class firing-without-effect
Matched 1
Ran 1
Effects denial 0, modification 0, consumed output 0
First ran at 2026-01-01T09:00:07Z
Last ran at 2026-01-01T09:00:07Z

### note-prompt.sh

Identifier UserPromptSubmit:0:0
Event UserPromptSubmit
Matcher None
Class firing-without-effect
Matched 1
Ran 1
Effects denial 0, modification 0, consumed output 0
First ran at 2026-01-01T09:00:00Z
Last ran at 2026-01-01T09:00:00Z

### wrap-up.sh

Identifier Stop:0:0
Event Stop
Matcher None
Class firing-without-effect
Matched 1
Ran 1
Effects denial 0, modification 0, consumed output 0
First ran at 2026-01-01T09:03:00Z
Last ran at 2026-01-01T09:03:00Z

## Near misses

None.
