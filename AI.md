# AI in this project

This document describes how AI tools were used in building Cookbook, what that means for the codebase, and what is expected of future contributors.

---

## Tools used

**[Claude Code](https://claude.ai/code)** (Anthropic) — used as the primary development tool throughout the project, starting from the initial architecture and continuing through v0.1.0.

No other AI tools were used in a significant capacity.

---

## How it was used

This project was built through **vibe coding** — a workflow where the human provides direction, goals, and feedback, and the AI implements. In practice this meant:

- The human described what they wanted (a feature, a fix, a refactor, a new file)
- Claude Code explored the codebase, proposed an approach, and wrote the code
- The human tested the result on real hardware (GTK on Linux, Pantryman on a Pixel 7)
- Bugs and regressions were reported back conversationally and fixed iteratively

Claude Code wrote the overwhelming majority of the source code in this repository. The human's contributions were direction, domain knowledge, real-device testing, and final judgement on what to keep.

This is not unusual for a solo project at this stage. The code was reviewed and understood (at least at a high level) before being accepted, and the human was able to reproduce and diagnose bugs independently.

---

## What this means for the codebase

- **The code is correct but not hand-polished.** There are places where a human author would have made slightly different style choices. That is acceptable.
- **The architecture is sound.** The `cookbook-engine` / frontend separation, the data format, and the sync model were all deliberate design decisions, not AI accidents.
- **Tests exist but coverage is incomplete.** The engine has a solid test suite. The GTK and Android layers are under-tested. See [TODO.md](TODO.md).
- **Some dead code and minor inconsistencies may remain.** AI-assisted projects accumulate small artefacts. If you find something odd, fix it and open a PR.

---

## Expectations for future contributors

**You are welcome to use AI tools.** There is no rule against it. However:

1. **Understand what you submit.** If an AI wrote a change, you are responsible for it. Read it. If you cannot explain what it does, do not submit it.

2. **Disclose significant AI involvement** in your PR description if the change is substantial. This is not a shaming mechanism — it is useful context for reviewers (e.g. "AI-generated, I reviewed and tested on device").

3. **Do not use AI to paper over problems you do not understand.** If something is broken, diagnose it first. An AI can help you fix a bug you understand; it should not be used to generate plausible-looking code until the tests go green.

4. **Security-sensitive code requires human review.** JNI unsafe blocks, file permissions, and URI permission grants should be reviewed by a human who understands the threat model, regardless of who wrote the initial draft.

---

## Trust boundaries

The AI-generated code in this repository operates within well-defined boundaries:

- **File I/O is limited to the user-chosen data directory.** The engine reads and writes only files it is explicitly pointed at.
- **No network access in the engine.** `cookbook-engine` is a pure file I/O library. All network activity (pCloud sync, etc.) is handled at the platform layer (Android SAF, or the user's own sync tool on desktop).
- **No telemetry, analytics, or outbound connections** are present or planned in any component.

If you find code that violates these boundaries, treat it as a bug and report it.
