# Rule Execution Order

This file defines the **mandatory execution order** for all Cline protocols and rules.
This is automatically loaded first for every task.

---

## MANDATORY EXECUTION SEQUENCE

### PHASE 1: INITIALIZATION (RUN FIRST - BEFORE ANY OTHER ACTION)
1.  ✅ **Read all `.clinerules` files** in the current working directory
2.  ✅ **Recursively list project files** to understand structure
3.  ✅ **Identify build systems, test frameworks, and languages**
4.  ✅ **Check for active `memory-bank/`** and load consolidated learnings

### PHASE 2: TASK EXECUTION
5.  Execute the user's requested task using appropriate tools

### PHASE 3: PRE-COMPLETION (RUN BEFORE `attempt_completion`)
6.  ✅ **Check for user feedback** - if feedback received, run `self-improving-cline.md` reflection protocol
7.  ✅ **Run Continuous Improvement Protocol** (always execute unless task is trivial)
8.  ✅ **Validate all task requirements** are met
9.  ✅ **Call `attempt_completion`** only after all above steps complete successfully

---

## PROTOCOL PRIORITY ORDER
1.  **Highest Priority:** `cline-continuous-improvement-protocol.md`
2.  **High Priority:** `self-improving-cline.md`
3.  **Normal Priority:** All workflow and task specific rules
4.  **Low Priority:** Language and technology guides

---

## INTEGRITY CHECKS
- Never skip execution order
- Never call `attempt_completion` before running pre-completion protocols
- Always log all learnings to `memory-bank/`