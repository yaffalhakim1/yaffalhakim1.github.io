---
title: "Structs & Enums: The \"Blueprint\" and \"State Machine\""
date: 2026-04-07
slug: structs-enums-the-blueprint-and-state-machine
description: "In JavaScript, you often use plain objects for everything. In Rust, we use Structs and Enums to model data with absolute precision. Think of it as the..."
tags: [rust, interfaces, enums, structs, union-types]
---



In JavaScript, you often use plain objects for everything. In Rust, we use **Structs** and **Enums** to model data with absolute precision. Think of it as the difference between a "Sketch" and an "Engineering Blueprint."

---

## 🏛️ 1. Structs: The "Blueprint" Contract
A **Struct** is the contract for the shape of your data. It is a strictly typed object that ensures every instance has the exact same fields and types.

```rust
struct User {
    active: bool,
    username: String,
    email: String,
}

fn main() {
    let mut user1 = User {
        email: String::from("someone@example.com"),
        username: String::from("someuser123"),
        active: true,
    };

    user1.email = String::from("another@example.com"); // ⬅️ Modifying the blueprint
}
```

> ### TIP: 🏗️ Build Faster with "Struct Updates"
> Similar to the JS spread operator (`...`), Rust has the "Struct Update Syntax" for creating new versions of your data:
> ```rust
> let user2 = User {
>     email: String::from("copy@example.com"),
>     ..user1 // ⬅️ Like { ...user1 } in JS
> };
> ```

---

## 🧠 2. The Implementation (`impl`): Separation of Logic
In JS classes, you put data and functions in the same box. In Rust, we separate the **Data (Struct)** from the **Logic (Implementation)**.

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // This function "signs" the rectangle, giving it new capabilities
    fn area(&self) -> u32 {
        self.width * self.height
    }
}
```

---

## 🥊 3. Enums: The "State Machine" Contract
While a Struct is about **"AND"** (I have a width AND a height), an Enum is about **"OR"** (I am EITHER a Success OR an Error).

In TypeScript, you'db use a "Discriminated Union":
```typescript
type Message = 
    | { type: 'quit' }
    | { type: 'move', x: number, y: number }
    | { type: 'write', text: string };
```

In Rust, Enums are much more powerful because they **carry data directly** and are checked by the compiler for exhaustiveness.

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
```

> ### IMPORTANT: The Magic: Pattern Matching
> You don't need a `type` or `kind` field. Use **`match`** to extract the data safely! The compiler will literally refuse to run your code if you forget to handle one of the states (e.g., forgetting the `Error` state in an API call).

---

## 💡 What these Unlock
- **Exhaustive Logic**: Using Enums ensures your application handles every possible state (like `Loading`, `Success`, `Error`) without "Undocumented States" causing bugs.
- **Data Integrity**: Structs ensure that your data remains in a "Known-Good" state throughout the entire application lifecycle.