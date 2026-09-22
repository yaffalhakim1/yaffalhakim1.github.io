---
title: "Error Handling: The \"Safety Net Contract\""
date: 2026-04-07
slug: error-handling-the-safety-net-contract
description: "In JavaScript, errors are \"invisible.\" You don't know if a function will return null, undefined, or throw a hidden exception until your code crashes i..."
tags: [error-handling, rust, result, panic, option]
---



In JavaScript, errors are "invisible." You don't know if a function will return `null`, `undefined`, or throw a hidden exception until your code crashes in production. 

In Rust, **Error Handling** is an explicit **Safety Net Contract**. There is no `null` and no `throw`. Instead, you "Wrap" your data in specialized containers that force you to handle failure cases before you can access the success data.

---

## 🏗️ The 3-Step Narrative

### 1. `Option<T>`: The "Null" Killer
JavaScript: `const x = findUser(id);` might return `null`. Rust: It returns an **Option**.
Think of an `Option` as a **Box** that is either **Full (Some)** or **Empty (None)**.

```rust
enum Option<T> {
    Some(T),
    None,
}

let some_number = Some(5);
let absent_number: Option<i32> = None; // ⬅️ You must be explicit if it's None
```
> ### IMPORTANT: The Rule
> To get the value out of the `Some` box, you **must** handle the `None` case. You can't just "forget" it exists.

### 2. `Result<T, E>`: The `try/catch` Replacement
Instead of throwing exceptions, Rust functions return a **Result**.
*   `Ok(T)` - Success with data.
*   `Err(E)` - Error with info.

```rust
use std::fs::File;

fn main() {
    let result = File::open("hello.txt");

    let file = match result {
        Ok(file) => file, // ⬅️ Success case
        Err(e) => panic!("File missing: {:?}", e), // ⬅️ Handle failure case
    };
}
```

### 3. The `?` Operator: The "Bubble" Contract
If you don't want to handle the error in the current function, you can use the **`?` operator** to "bubble" it up to the caller. 

```rust
fn read_username() -> Result<String, io::Error> {
    let mut s = String::new();
    File::open("hello.txt")?.read_to_string(&mut s)?; // 🪄 Bubbling up errors
    Ok(s)
}
```

---

## 🔬 From the Official Book

### 🚫 `panic!`: The "Emergency Stop"
When Rust encounters a truly unrecoverable error, it **Panics.** This is like your program pulling its own emergency brake. Use this sparingly, only for things that should "never" happen.

```rust
fn main() {
    panic!("crash and burn"); // 💥 Everything stops here
}
```

---

## 💡 What Error Handling Unlocks
- **Compile-Time Safety**: You can no longer have "null pointer exceptions" because there is no `null`.
- **Predictable Flow**: Every error becomes a known value that you can handle using pattern matching.
- **Maintainable Code**: When you see `?` in a function, you know exactly where the failure points are without searching for `try/catch` blocks.
- **Better API Design**: By returning `Result` and `Option`, your function names become honest about whether they might fail.