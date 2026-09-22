---
title: "Generics: The \"Template Contract\""
date: 2026-04-07
slug: generics-the-template-contract
description: "If you are a frontend developer coming from TypeScript, you actually already understand the core concept of Generics in Rust! They solve the exact sam..."
tags: [frontend, typescript, rust, generics]
---



If you are a frontend developer coming from TypeScript, you actually already understand the core concept of Generics in Rust! They solve the exact same problem in both languages: **writing reusable code that handles multiple types without sacrificing safety.**

Think of a **Generic** as a **Placeholder in a Contract.** You are saying: "I will accept ANY data type here, provided it follows the rules I define later."

---

## 🏗️ The 3-Step Narrative

### 1. The Generic Placeholder
Imagine you want a function that just returns whatever you pass into it. Instead of writing separate functions for every single type (`i32`, `String`, etc.), you use a placeholder: `<T>`.

```rust
// Rust
fn identity<T>(arg: T) -> T {
    arg
}

let num = identity(42); // Rust infers <i32> automatically!
```
> ### NOTE: Parity with TypeScript
> This is exactly like: `function identity<T>(arg: T): T { return arg; }`.

### 2. The Monomorphization (The "Free" Performance)
This is where Rust pulls ahead in performance. In **TypeScript**, generics are erased after the code is built (Type Erasure). In **Rust**, generics exist to write highly-optimized machine code.

During compilation, Rust uses a process called **Monomorphization**. It looks at your code, sees you called `identity` with an integer AND a string, and actually writes **two separate, highly-optimized functions** for you behind the scenes.

```rust
// What the compiler actually generates:
fn identity_i32(arg: i32) -> i32 { arg }
fn identity_str(arg: &str) -> &str { arg }
```
Because of this, there is **zero runtime overhead** when using generics.

### 3. Combining with Traits: "Trait Bounds"
Generics become truly "Smart" when you combine them with ****Traits (Interfaces)****. You can say: "I'll accept ANY type `<T>`, as long as it has signed the `Summary` contract."

```rust
pub fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}
```

---

## 🔬 From the Official Book

### Generic Structs & Enums
The standard library uses generics for its most famous data structures.

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

---

## 💡 What Generics Unlock
- **Code Reusability**: Write one function that handles all your data types perfectly.
- **Zero-Cost Abstractions**: Get the flexibility of a dynamic language with the performance of a manual one.
- **Strict Safety**: The compiler ensures that a generic type never "breaks the rules" of the contract it's placed in.