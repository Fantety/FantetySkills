---
name: build-user-centered-products
license: MIT
description: Apply user-centered product reasoning while designing, implementing, modifying, or reviewing user-facing software. Use for product and software development tasks where must translate requests into coherent UX, prioritize the primary user goal, reason across interface states and transitions, make proportional value-versus-complexity tradeoffs, choose context-appropriate behavior, and validate the end-to-end experience instead of satisfying requirements only literally or locally.
---

# Build User-Centered Products

Treat each request as evidence of a user need, not a complete product specification. Preserve explicit constraints while optimizing for the user's real goal and the coherence of the whole experience.

## Form a Product Model

Before making consequential decisions:

1. Identify the user, their goal, and meaningful success.
2. Identify the primary task. Allocate attention, space, speed, and friction according to its importance and frequency.
3. Model the relevant user and system states, their transitions, and the useful next action in each.
4. Evaluate the full journey. Reject locally correct behavior that makes the overall task confusing, fragile, or hard to recover.

Keep this reasoning internal unless an assumption or tradeoff materially affects the result.

## Decide by Meaning and Proportion

- Treat components, patterns, APIs, and technical requirements as means. Choose behavior from the situation's meaning and scope, not a blanket rule.
- Translate system conditions into user meaning and a useful next action. Preserve effort and help the user resume the original intent.
- Make hierarchy reflect user value. Apply consistency only to equivalent situations.
- Resolve ambiguity with the smallest user-serving assumption. Avoid UI, explanation, and ceremony that do not help users decide, act, understand, or recover.

Evaluate every refinement independently. Compare expected user value—reach, frequency, severity, and effect on task completion—with total system cost—complexity, coupling, regression risk, maintenance, and opportunity cost. Prefer the simplest change that captures most value. Accept minor imperfection when removing it has negligible user impact but disproportionate cost. Spend complexity where it protects task completion, effort, trust, safety, or recoverability.

## Implement and Verify

Inspect the surrounding flow. Implement the smallest coherent change across the layers required by the intended experience. Separate technical diagnostics from user-facing meaning and keep enough structured state for appropriate response and recovery.

Test representative journeys and transitions, not only isolated handlers or the happy path.

Before finishing, ask:

- Is the main goal obvious and easy to pursue?
- Does each important state offer an understandable next step and preserve continuity?
- Are prominence, interruption, friction, and implementation complexity proportional to user value?
- Does the journey remain coherent without knowledge of the implementation?

If not, revise product behavior rather than merely polishing presentation.
