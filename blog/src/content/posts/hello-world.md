---
title: "Hello World"
description: "My first blog post - an introduction to this blog and what to expect."
date: 2024-11-22
tags: ["intro", "meta"]
---

Welcome to my blog! This is my first post.

## What to Expect

I'll be writing about:

- **Rust** - Systems programming, async patterns, FFI
- **Distributed Systems** - CQRS, Event Sourcing, consistency patterns
- **Backend Development** - PostgreSQL optimization, API design, performance

## Code Examples

Here's a quick Rust example:

```rust
async fn process_event(event: Event) -> Result<(), Error> {
    let handler = EventHandler::new();
    handler.apply(event).await?;
    Ok(())
}
```

Stay tuned for more posts!
