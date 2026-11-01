# hookdoctor

A doctor for hooks, which fire, which never fire, which fire and change nothing, and which are clinically dead, with the examination shown.

## What this is

Hook tooling logs that a hook fired and checks that it is installed, and never joins a denial to an outcome, so a hook that has never fired and a hook that blocks a disaster weekly receive the same audit verdict. Hookdoctor diffs configured matchers against execution traces from session files and classifies each hook as firing, silent, or firing without effect. Near misses are reported separately, so a rare fire alarm is not mistaken for a corpse. Its verdicts join the ledger as evidence, the same as every other check's. The boundary with its sibling is one sentence. Hookdoctor audits the harness's hooks, blindgate audits CI's checks.

## Status

Seeded in October 2026. The build has not started. Part of the impatient programming portfolio, in the Gate Truth arc.

## Licence

MIT. See LICENSE.
