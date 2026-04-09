# personal-dev-workflow

A sandbox for practicing **Spec-Driven Development** ([pattern reference](https://github.com/PaulDuvall/ai-development-patterns?tab=readme-ov-file#spec-driven-development)).

The vehicle for the practice is `vibe-board/` — a small local kanban web app written in Rust. The *product* is not the point. The *loop* is:

```
draft spec  →  write acceptance tests  →  implement until tests pass  →  mark spec done
```

Every code change traces back to a file in `specs/`. See [`specs/README.md`](specs/README.md) for the format and lifecycle.

## Layout

- `specs/` — one file per feature, with Given/When/Then acceptance criteria
- `vibe-board/` — the Rust app (axum + sqlx + askama + htmx)

## Running

```
cd vibe-board
cargo test    # runs all spec acceptance tests
cargo run     # starts the server on :3000
```
