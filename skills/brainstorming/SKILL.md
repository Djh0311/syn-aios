---
name: brainstorming
description: Use for ambiguous new feature, product, component, functionality, behavior, or creative design work where intent, options, or acceptance need clarification before implementation
---

# Brainstorming Ideas Into Designs

## Overview

Help turn ideas into fully formed designs and specs through natural collaborative dialogue.

Start by understanding the current project context, then ask questions one at a time to refine the idea. Once you understand what you're building, present the design in small sections (200-300 words), checking after each section whether it looks right so far.

## The Process

**Understanding the idea:**
- Check out the current project state first (files, docs, recent commits)
- Ask questions one at a time to refine the idea
- Prefer multiple choice questions when possible, but open-ended is fine too
- Only one question per message - if a topic needs more exploration, break it into multiple questions
- Focus on understanding: purpose, constraints, success criteria

**Exploring approaches:**
- Propose 2-3 different approaches with trade-offs
- Present options conversationally with your recommendation and reasoning
- Lead with your recommended option and explain why

**Presenting the design:**
- Once you believe you understand what you're building, present the design
- Break it into sections of 200-300 words
- Ask after each section whether it looks right so far
- Cover: architecture, components, data flow, error handling, testing
- Be ready to go back and clarify if something doesn't make sense

## After the Design

**Documentation:**
- For Standard or Strict Path work in an installed project, write the validated design to `docs/plans/YYYY-MM-DD-<topic>-design.md` when a durable plan is useful.
- For work on the standard rule source package itself, write durable designs to `plans/YYYY-MM-DD-<topic>-design.md`; do not create source-package runtime state under `docs/**`.
- Include path, read/write scope, acceptance criteria, verification, and unresolved questions.
- Do not commit automatically; commits require explicit user confirmation.

**Implementation (if continuing):**
- Ask: "Ready to set up for implementation?"
- Use `using-git-worktrees` only when isolation is required and Git/worktrees are available.
- Use `writing-plans` to create a detailed implementation plan when work is multi-step.

## Key Principles

- **One question at a time** - Don't overwhelm with multiple questions
- **Multiple choice preferred** - Easier to answer than open-ended when possible
- **YAGNI ruthlessly** - Remove unnecessary features from all designs
- **Explore alternatives** - Always propose 2-3 approaches before settling
- **Incremental validation** - Present design in sections, validate each
- **Be flexible** - Go back and clarify when something doesn't make sense
