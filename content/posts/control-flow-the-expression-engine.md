---
title: "Control Flow: The \"Expression Engine\""
date: 2026-04-07
slug: control-flow-the-expression-engine
description: "In JavaScript, if/else is a Statement (it does something). In Rust, if/else and match are Expressions (they evaluate to a value). Think of Rust's co..."
tags: [rust, loops, if-else, match, control-flow]
---



In JavaScript, `if/else` is a **Statement** (it does something). In Rust, `if/else` and `match` are **Expressions** (they evaluate to a value). 

Think of Rust's control flow as a **Logic Engine** that processes conditions and returns direct results, acting more like a "Turn-Key" switch than a simple "Yes/No" gate.

---

## 🏗️ The 3-Step Narrative

### 1. `if/else` as Data
In JS, you use `const x = condition ? a : b`. In Rust, the entire `if` block is the value itself.

```rust
fn main() {
    let condition = true;
    let number = if condition { 5 } else { 6 }; // 🪄 if/else is the value

    println!("The value is: {number}");
}
```
> ### IMPORTANT: The Rule
> Because it's a typed contract, both the `if` and the `else` blocks **must** return the same data type. You can't return a number in one and a string in the other.

### 2. `match`: The "Exhaustive" Engine
While JS uses `switch`, Rust uses **`match`**. This is a strict **Exhaustiveness Contract**. The compiler literally forces you to handle every possible case (including `_` for "anything else").

```rust
fn main() {
    let number = 3;
    
    match number {
        1 => println!("One!"),
        2 | 3 | 5 | 7 | 11 => println!("This is a prime"),
        13..=19 => println!("A teen"),
        _ => println!("Something else"), // 🔮 Exhaustive default
    }
}
```
> ### NOTE: The Magic
> This exhaustiveness is a superpower when combined with ****Structs and Enums****. It prevents the "Unhandled State" bugs that often crash frontend apps (like forgetting a `Loading` or `Error` state).

### 3. Loop Power: Returning Data from Loops
Rust takes loops to a whole new level. You can actually **return a value** directly from a loop once it breaks!

```rust
fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2; // 🪄 loop evaluates to 20
        }
    };
}
```

---

## 🔬 From the Official Book

### Pattern Matching with `if let`
Sometimes a full `match` is too much boilerplate. Use **`if let`** as a "Shorthand Contract" when you only care about one specific pattern.

```rust
if let Some(3) = some_value {
    println!("Found three!");
}
```

---

## 💡 What Control Flow Unlocks
- **Unbreakable Logic**: By forcing exhaustiveness in `match`, Rust eliminates the "Ghost States" that cause logic bugs in your application.
- **Syntactic Sugars**: Expression-based `if/else` means cleaner, more declarative code without needing ternary operators or temporary variables.
- **Infinite Flexibility**: Loops that return values allow for sophisticated "Retry" or "Calculation" patterns that are impossible in standard JS loops.