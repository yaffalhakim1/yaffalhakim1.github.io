---
title: "Variables: The \"Mutability Contract\""
date: 2026-04-07
slug: variables-the-mutability-contract
description: "In JavaScript, you use let and const for everything. In Rust, variables are Immutable by Default. This is a strict Mutability Contract that ensures yo..."
tags: [variables, rust, constants, shadowing, mutability]
---



In JavaScript, you use `let` and `const` for everything. In Rust, variables are **Immutable by Default**. This is a strict **Mutability Contract** that ensures your data doesn't change unless you explicitly "sign" for it.

---

## 🏗️ The 3-Step Narrative

### 1. Immutable by Default (The Safety Net)
In JS, you might use `let` for things that never change. In Rust, you **must** use `let` for things that are "locked."

```rust
fn main() {
    let x = 5;
    // x = 6; ❌ Error: cannot assign twice to immutable variable `x`
}
```

### 2. Signing for Change (`mut`)
If you want to be able to change a value, you must explicitly sign the **Mutability Contract** using the **`mut`** keyword.

```rust
fn main() {
    let mut x = 5;
    println!("The value of x is: {x}"); // 5
    x = 6; // ✅ Signed for change
    println!("The value of x is: {x}"); // 6
}
```

### 3. Shadowing (The "Re-Declaration" Contract)
In JS, re-declaring a variable is a bug. In Rust, it's a powerful feature called **Shadowing**. It allows you to transform a value (e.g., from a `String` to a `Length`) while reusing the same name.

```rust
fn main() {
    let spaces = "   "; // ⬅️ A string
    let spaces = spaces.len(); // 🪄 Shadowing: now it's an integer
}
```
> ### NOTE: The Magic
> Shadowing is different from `mut`. Because you are using `let` again, you are effectively creating a **new contract** for the same name, allowing you to change the **Type** of the data as well.

---

## 🔬 From the Official Book

### Constants vs Variables
Constants aren't just "immutable variables." They are **Compile-Time Contract**. They must have a type annotation, can't be shadowed, and are evaluated before the program even runs.

```rust
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
```

---

## 💡 What Variables Unlock
- **Memory Safety**: By being immutable by default, Rust prevents accidental "side-effect" bugs where a value changes without you realizing it.
- **Type Flexibility**: Shadowing allows you to keep your variable names clean while your data undergoes transformations.
- **Performance**: Because constants are evaluated at compile-time, they add **zero** runtime overhead to your application.
- **Readability**: When you see `mut`, you know exactly where the "Moving Parts" of your logic are.