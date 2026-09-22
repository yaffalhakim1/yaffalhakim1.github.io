---
title: "Functions: The \"Expression Narrative\""
date: 2026-04-07
slug: functions-the-expression-narrative
description: "In JavaScript, functions often feel like a playground of \"statements\" (lines of code that just do stuff). In Rust, functions are structured as a Narra..."
tags: [functions, rust, expressions, statements, implicit-returns]
---



In JavaScript, functions often feel like a playground of "statements" (lines of code that just do stuff). In Rust, functions are structured as a **Narrative of Expressions.** 

Almost everything in Rust is an **Expression** (it evaluates to a value). This means your functions don't always need a `return` keyword—they just "resolve" to their last line.

---

## 🏗️ The 3-Step Narrative

### 1. Declaring the Contract (Parameters and Types)
In JS, you can omit types. In Rust, you **must** sign the contract by declaring the types of your parameters and your return value (`->`).

```rust
fn main() {
    let x = another_function(5);
    println!("The value is: {x}");
}

// ⬅️ Parameter and Return Types are Required
fn another_function(x: i32) -> i32 {
    x + 1
}
```

### 2. The "Expression" vs "Statement" Story
This is the "Aha!" moment for frontend developers.
* **Statement**: A line that ends in a semicolon (`;`). It does a task but returns **nothing** (unit `()`).
* **Expression**: A line that **doesn't** end in a semicolon. It evaluates to a **value**.

```rust
fn main() {
    let y = {
        let x = 3;
        x + 1 // ⬅️ No semicolon! This is an expression.
    }; // y is now 4
}
```

### 3. The Implicit Return (The Clean Contract)
Most Rust functions use their last line as the return value by simply omitting the semicolon.

```rust
fn five() -> i32 {
    5 // ✨ No return keyword needed!
}
```
> ### NOTE: The Magic
> If you add a semicolon to that `5`, it becomes a **Statement** and returns nothing, which would break the `-> i32` contract.

---

## 🔬 From the Official Book

### Snakes on a Case
Rust uses **snake_case** for function names by convention, unlike the **camelCase** used in JavaScript.

```rust
fn my_cool_function() {} // Correct
// fn myCoolFunction() {} // Incorrect
```

---

## 💡 What Functions Unlock
- **Clean Syntax**: No more "return" keyword bloat. The last line of your function is the "Final Result" of the narrative.
- **Strict APIs**: By requiring parameter and return types, your functions become **Self-Documenting Contracts**.
- **Self-Cleaning Blocks**: The use of curly-brace expressions means you can create "mini-narratives" anywhere in your code to scope temporary variables.