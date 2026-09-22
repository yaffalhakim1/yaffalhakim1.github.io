---
title: "Testing: The \"Quality Contract\""
date: 2026-04-07
slug: testing-the-quality-contract
description: "Coming from a background of Jest, Vitest, or Mocha, you'll find Rust's built-in testing suite incredibly refreshing. No extra dependencies are needed ..."
tags: [unit-test, testing, rust, jest, vitest]
---



Coming from a background of **Jest**, **Vitest**, or **Mocha**, you'll find Rust's built-in testing suite incredibly refreshing. No extra dependencies are needed for basic unit testing!

Think of Rust Testing as a **Quality Contract**. By writing tests, you are "Signing" for the behavior of your code, ensuring that it always meets the expectations you've defined, even as the project grows.

---

## 🏗️ The 3-Step Narrative

### 1. The Unit Test (The "Instant Feedback" Contract)
In JS, you install `jest` and write `test()`. In Rust, you use the **`#[test]`** attribute. 
You run your tests with **`cargo test`**.

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn exploration() {
        let result = 2 + 2;
        assert_eq!(result, 4); // 🪄 The "Expect" Contract
    }
}
```
> ### NOTE: The Magic
> The `#[cfg(test)]` attribute tells Rust to **only compile the test code** when running `cargo test`, keeping your production binary lean and fast.

### 2. The Integration Test (The "System" Contract)
While Unit Tests focus on a single piece of code, **Integration Tests** sign the contract for the **entire crate**. You place these in a dedicated **`tests/`** directory at your project root. 
*   **What it unlocks:** The ability to see how different parts of your library work together as a single system.

### 3. The Documentation Test (The "Living" Contract)
This is the "Superpower" of Rust. You can write your tests **inside your Markdown documentation** (within `///` comments)! 
*   **What it unlocks:** Rust will run these examples during every `cargo test`, ensuring that your documentation **never** becomes out-of-date or broken.

---

## 🔬 From the Official Book

### Specialized Assertions
Think of these as exactly like your favorite JS `expect()` calls:
*   **`assert_eq!(x, y)`**: Like `expect(x).toBe(y)`.
*   **`assert_ne!(x, y)`**: Like `expect(x).not.toBe(y)`.
*   **`assert!(x)`**: Like `expect(x).toBeTruthy()`.

### Testing for Panics
If you want to ensure your code throws an error (panics) under certain conditions:
```rust
#[test]
#[should_panic(expected = "less than or equal to 100")]
fn greater_than_100() {
    Guess::new(200);
}
```

---

## 💡 What Testing Unlocks
- **Zero-Dependency Quality**: You don't need a single configuration file or extra package to start testing your Rust library.
- **Built-in Performance**: No slow startup times like some JS testing frameworks—Rust tests are compiled directly into machine code.
- **Self-Documenting Code**: Doc-Tests ensure that any example you share with another developer is **guaranteed** to work.
- **Fearless Refactoring**: If your tests pass, you have the safety of knowing you haven't broken the core "Contract" of your code.