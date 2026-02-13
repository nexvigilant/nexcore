# Delegation Routing Training Materials
## CEP Stage 2 - Externalization Complete

**Date:** 2026-02-01  
**Status:** Final  
**Target:** Gemini Flash training via REST API flight simulator

---

## Quick Start

### For Learning the Basics
Start with: **DELEGATION_PRIMITIVES_SUMMARY.md**
- 5 primitives on one page
- Decision tree at a glance
- 3 fatal pitfalls
- 15 minutes to understand the fundamentals

### For Comprehensive Training
Read: **DELEGATION_TRAINING_CURRICULUM.md**
- Full 622-line curriculum
- 5 primitives with detailed explanations
- 10 training scenarios (easy → hard)
- Confidence scoring mechanics
- Real API endpoints for practice
- 1-2 hours for complete mastery

---

## The 5 Delegation Primitives (Summarized)

| Primitive | Type | Signal | Example |
|-----------|------|--------|---------|
| **Volume** | Axis (0→∞) | Higher counts → Faster models | 112 items → Flash |
| **Repetitiveness** | Binary | Same op repeated = Delegatable | Docstrings yes, Security logs no |
| **Structure** | Binary | Clear patterns = Automation-friendly | AAA pattern yes, Anomaly analysis no |
| **Reasoning** | Binary | Deep analysis needed = Keep Claude | Generate tests no, Improve coverage yes |
| **Risk** | Scale (Low→Critical) | Error impact = Priority to keep local | Security audit = Always Opus |

---

## The 6-Rule Decision Tree

```
1. Sensitive OR Critical?        → ClaudeOpus (veto)
2. Bulk (>10) + Repetitive + Structure? → GeminiFlash
3. Novel + Reasoning?            → ClaudeOpus
4. Multimodal?                   → GeminiPro
5. High Volume (>50)?            → GeminiFlash
6. Default (no signals)          → ClaudeSonnet
```

**First match wins.** Apply rules in priority order.

---

## REST API Flight Simulator

Practice with real endpoints:

### 1. Get Training Suite
```bash
curl http://localhost:3030/api/v1/delegation/training
```

Returns all 10 scenarios with instructions and scoring.

### 2. Route a Task
```bash
curl -X POST http://localhost:3030/api/v1/delegation/route \
  -H "Content-Type: application/json" \
  -d '{
    "item_count": 112,
    "is_repetitive": true,
    "has_structure": true,
    "needs_reasoning": false,
    "is_novel": false,
    "is_sensitive": false,
    "is_multimodal": false,
    "error_cost": "low"
  }'
```

Returns: Model, Confidence (0-100%), Rationale, Prompt Hints

### 3. Validate Your Answer
```bash
curl -X POST http://localhost:3030/api/v1/delegation/validate \
  -H "Content-Type: application/json" \
  -d '{
    "scenario_id": 1,
    "predicted_model": "GeminiFlash"
  }'
```

Returns: Correct (true/false), Feedback, Score

---

## 10 Training Scenarios

### Easy (Foundation - 5 scenarios)

**1.1: Pure Bulk Work**
- 112 items, repetitive, structured, no reasoning, low risk
- Expected: GeminiFlash
- Score: 10 points

**1.2: Pure Novel Reasoning**
- 1 item, novel, needs reasoning, high risk
- Expected: ClaudeOpus
- Score: 10 points

**1.3: Pure Sensitivity**
- Sensitive, critical error cost
- Expected: ClaudeOpus
- Score: 10 points

**1.4: Pure Multimodal**
- 20 items, images, repetitive, structured
- Expected: GeminiPro
- Score: 10 points

**1.5: Default (No Signals)**
- 1 item, no strong signals
- Expected: ClaudeSonnet
- Score: 10 points

### Medium (Conflict Resolution - 3 scenarios)

**2.1: Bulk But No Reasoning**
- 20 items, repetitive, structured, medium error cost
- Expected: GeminiFlash
- Score: 20 points

**2.2: Novel But Bulk**
- 100 items, novel, repetitive, structured
- Expected: GeminiFlash
- Score: 20 points

**2.3: Multimodal With Reasoning**
- 50 medical images, needs reasoning
- Expected: GeminiPro
- Score: 20 points

### Hard (Edge Cases & Veto - 2 scenarios)

**3.1: Hidden Sensitivity Veto**
- 50 items, bulk signals BUT sensitive + critical
- Expected: ClaudeOpus (Rule 1 veto)
- Score: 30 points

**3.2: Volume Paradox**
- 500 items, bulk structure BUT high error cost (safety)
- Expected: ClaudeOpus (Rule 1 veto)
- Score: 30 points

**Total Possible Score: 170 points**

---

## Confidence Scoring Formula

```
Total = (Pattern × 0.4) + (ItemCount × 0.4) + (ErrorTolerance × 0.2)
```

**Thresholds:**
- > 70%: Safe to delegate
- 50-70%: Delegate with review
- < 50%: Keep with Claude
- 0% (Risk Veto): Never delegate

---

## Certification Rubric

| Score | Level | What It Means |
|-------|-------|-------------|
| 150+ | Expert | Autonomous delegation qualified |
| 130-150 | Advanced | Delegation with code review |
| 100-130 | Proficient | Can delegate with supervision |
| 70-100 | Competent | Learn more before autonomous |
| <70 | Novice | Review fundamentals and retry |

**Passing:** 140+/170 points

---

## 3 Fatal Pitfalls (Learn These!)

### Pitfall 1: "It's Repetitive, So Delegate"
**Wrong:** Repetitive alone = OK to delegate  
**Right:** Need (Repetitive + Structured + No Reasoning)

Example: "Review 10 security logs for threats"
- Repetitive? Yes
- But: Sensitive + needs reasoning
- Route to: ClaudeOpus (NOT Flash)

### Pitfall 2: "High Volume Saves Everything"
**Wrong:** 200 items = Always delegate  
**Right:** Check risk FIRST, volume second

Example: "Process 200 medical images for signals"
- High volume? Yes
- But: Critical medical decisions
- Route to: ClaudeOpus (NOT Flash)

### Pitfall 3: "Novel = Always Opus"
**Wrong:** Novel work always needs Opus  
**Right:** Novel + Bulk + Structured can still delegate

Example: "Generate 100 test cases for new protocol"
- Novel? Yes
- But: Repetitive (same test template), structured
- Route to: GeminiFlash (despite novelty)

---

## Model Profiles

### GeminiFlash
- **Strength:** Bulk Generation
- **Best For:** 100+ items, repetitive, structured, low-risk
- **Error Tolerance:** 80% (high)
- **Confidence Threshold:** 50%+

### GeminiPro
- **Strength:** Multimodal
- **Best For:** Images, documents, vision analysis
- **Error Tolerance:** 40% (medium)
- **Confidence Threshold:** 60%+

### ClaudeOpus
- **Strength:** Deep Reasoning
- **Best For:** Novel problems, sensitive, critical, architecture
- **Error Tolerance:** 10% (low)
- **Confidence Threshold:** Always route high-stakes here

### ClaudeSonnet
- **Strength:** Balanced
- **Best For:** Default, code gen, small refactoring
- **Error Tolerance:** 30% (medium)
- **Confidence Threshold:** 40%+ (default safety net)

### ClaudeHaiku
- **Strength:** Speed
- **Best For:** Simple queries, classification
- **Error Tolerance:** 50% (medium-high)
- **Confidence Threshold:** 60%+ for simple tasks

---

## How to Use This Training

### Stage 1: Learn (30 minutes)
1. Read DELEGATION_PRIMITIVES_SUMMARY.md
2. Understand the 5 primitives
3. Learn the 6 decision rules
4. Memorize the 3 fatal pitfalls

### Stage 2: Practice (60 minutes)
1. Call GET /api/v1/delegation/training
2. Review all 10 scenarios
3. Read explanations for easy ones
4. Study hard scenario logic

### Stage 3: Test (30 minutes)
1. Route 5 scenarios using the API
2. Check feedback via POST /validate
3. Score yourself
4. Identify weak areas

### Stage 4: Reflect
1. Did you miss any signals?
2. Were you over-confident?
3. What patterns tripped you up?
4. Ready for autonomous delegation?

---

## Key Takeaways

1. **Risk is a Veto** - Sensitive or Critical error cost always routes to Opus
2. **Combination Matters** - Repetitive alone ≠ delegatable (need structure + no reasoning too)
3. **Confidence is Probabilistic** - Not all routing decisions are 100% certain
4. **Edge Cases Reveal Truth** - Hard scenarios teach more than easy ones
5. **Real Examples > Abstract Rules** - You'll learn more from "wrong vs right" cases

---

## Architecture Reference

| Component | Location | Purpose |
|-----------|----------|---------|
| Core Router | `crates/nexcore-vigilance/src/primitives/delegation/routing.rs` | Routing logic (T1: Sequence of Mappings) |
| Confidence | `crates/nexcore-vigilance/src/primitives/delegation/confidence.rs` | Multi-dimensional scoring |
| Models | `crates/nexcore-vigilance/src/primitives/delegation/model.rs` | Model definitions + capabilities |
| Review | `crates/nexcore-vigilance/src/primitives/delegation/review.rs` | 5-phase validation protocol |
| Classification | `crates/nexcore-vigilance/src/primitives/delegation/classification.rs` | Classification tree primitive |
| API | `crates/nexcore-api/src/routes/delegation.rs` | REST endpoints + training suite |
| Demo | `crates/nexcore-vigilance/examples/delegation_demo.rs` | Live routing demonstration |
| Hook | `crates/nexcore-hooks/bin/delegation_loop.rs` | Delegation detection hook |

---

## What's Next After Certification?

Once certified (140+/170):

1. **You can autonomously route tasks** to the optimal model
2. **You understand the primitives deeply** - can apply to new scenarios
3. **You're ready for Stage 3 (DECOMPOSE)** - break routing into T1 primitives
4. **You can teach others** - explain decision logic confidently

---

## Questions & Feedback

These materials were created via CEP Stage 2 (SPEAK - Externalization).

If something is unclear:
1. Check the quick summary first
2. Read the full curriculum
3. Study the "wrong vs right" examples
4. Practice with real scenarios

---

## Attribution

**Created by:** Matthew Campion, PharmD; NexVigilant  
**Methodology:** Constructive Epistemology Pipeline (CEP)  
**Stage:** 2/8 - SPEAK (Externalization)  
**Date:** 2026-02-01  
**Status:** Complete - Ready for Gemini training

**Commit:** adcf190  
**Branch:** main

