# Improve RAG Scoring for Supervisor Matching

## Problem

The current embedding-based similarity search returns suboptimal matches between student queries and lecturer profiles. Two root causes:

1. **Format asymmetry**: The query template and storage template use different structures, headers, and terminology — forcing the embedding model to bridge an unnecessary semantic gap.
2. **No reranking**: The top-K results from Qdrant embedding similarity are returned directly. In RAG systems, dense retrieval is typically followed by a reranking step (cross-encoder or LLM scorer) to improve precision.

## Current State

### Storage template (seed.rs)
```
# LECTURER: {name}
## Biography:
{biography}
## Area of Expertise:
{areas_of_expertise}
```

### Query template (main.rs)
```
Area of interest:
{interesting_topics}

Additional text:
{additional_text}

Based on the given information, who is the most suitable supervisor?
```

Key mismatches:
- `Area of interest` vs `Area of Expertise` — different terminology
- `Additional text` vs `Biography` — different field names
- Query has an instructional sentence at the end — embedding models encode this as noise, pulling the query vector away from documents
- Query has no `# LECTURER:`-equivalent header — no structural symmetry

### Note on the instructional sentence and Qdrant docs

The Qdrant external inference docs show `Document.text` being passed directly to an embedding model. The `Document` struct is a **data carrier**, not an instruction interface. Qdrant sends the raw text to OpenRouter's embedding API, which returns a vector. The embedding model (e.g., `qwen3-embedding-8b`) encodes semantic meaning — it does NOT interpret instructions. The phrase "Based on the given information, who is the most suitable supervisor?" is embedded as part of the query vector alongside the actual student interests, diluting the signal. The official Qdrant examples confirm this pattern: the query text is purely descriptive (`"How to bake cookies?"`), with no instructions.

---

## Plan

### Step 1: Align query template with storage template

**File**: `apps/backend/src/main.rs`

Change the query TEMPLATE to structurally mirror the storage template:

```
# STUDENT

## Biography:
{additional_text}

## Area of Expertise (seeking):
{interesting_topics}
```

Rationale:
- `# STUDENT` mirrors `# LECTURER` — creates structural symmetry
- `Biography` matches `Biography` — same field name
- `Area of Expertise (seeking)` near-matches `Area of Expertise` — embedding model treats adjacent tokens similarly
- Removed the instructional sentence — eliminates embedding noise
- `additional_text` maps to Biography (`Biography that the student prefers` isn't needed; "Biography" is enough)

### Step 2: Add LLM-based reranking after Qdrant retrieval

**File**: `apps/backend/src/main.rs`

After Qdrant returns the top-K results, send them to an LLM for reranking.

Flow:
1. Retrieve top-5 (increase `limit` from 2 to 5) from Qdrant
2. Build a prompt containing: the student's query + all 5 candidate lecturer profiles with their metadata
3. Ask the LLM to rank them by suitability and return ordered results
4. Return the reranked list

New dependency: add OpenAI-compatible HTTP client (or reuse `reqwest`) to call OpenRouter's chat API.

**Note**: Since field names in the query template change, the `PickSupervisorRequest` struct field names should remain unchanged (`interesting_topics` and `additional_text` still make sense as API field names) — only the internal template changes.

### Step 3: (Optional stretch goal) Hybrid search

If Steps 1-2 are still insufficient, add sparse/BM25 keyword search alongside dense embeddings. Qdrant supports hybrid search natively. This helps when expertise area keywords (e.g., "machine learning", "database") appear explicitly in both query and documents.

But do this only if Steps 1-2 don't yield sufficient improvement.

---

## Files to modify

| File | Change |
|------|--------|
| `apps/backend/src/main.rs` | Update `TEMPLATE`, increase `limit`, add reranking logic |
| `apps/backend/Cargo.toml` | Possibly add `reqwest` feature for JSON streaming if reranker needs it (already available) |

## Validation

- Run `cargo test` to ensure existing tests pass
- Run `cargo fmt` and `cargo clippy` per AGENTS.md guidelines
- Manual test: hit `/supervisors/pick` endpoint and verify reranked results are ordered and relevant
