---
title: "Modules & Organization"
date: 2026-04-07
slug: modules-organization
description: "Rust's modules are similar to JS/TS files and folders, but with much more explicit control over visibility. > ABSTRACT: 🧩 Analogy for Frontend Deve..."
tags: [organization, modules, rust, exports, imports]
---



Rust's modules are similar to JS/TS files and folders, but with much more explicit control over visibility.

> ### ABSTRACT: 🧩 Analogy for Frontend Developers
> - **`mod` is like a file or folder**: It defines a boundary of code.
> - **`pub` is like `export`**: It makes an item available to the outside world.
> - **`use` is like `import`**: It brings an item into your current scope.

---

## 🏗️ `mod` and `pub` (The JS `export`)

In JS, everything in a file is "internal" until you `export` it. In Rust, everything is **private by default**.

```rust
mod my_module {
    pub fn public_fn() {} // ⬅️ This is your JS `export`
    fn private_fn() {}    // ⬅️ This is an internal function
}
```

> ### NOTE: Module Structure
> - **Inline**: `mod name { ... }` (Great for tests or small components).
> - **Separate Files**: Create a file named `my_module.rs` and then in `main.rs`, use `mod my_module;`.

---

## 🎨 The Module Tree

Rust organizes code like a filesystem. 

```rust
// src/lib.rs (Crate Root)
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

pub fn eat_at_restaurant() {
    // Absolute Path (starts with `crate::`)
    crate::front_of_house::hosting::add_to_waitlist();

    // Relative Path (starts with module name)
    front_of_house::hosting::add_to_waitlist();
}
```

> ### WARNING: ⚠️ Privacy Rules
> A parent cannot see into a child's private items, but a child can see into its parent's items using **`super`**.
> ```rust
> mod back_of_house {
>     fn fix_incorrect_order() {
>         super::deliver_order(); // ⬅️ Accessing parent (like `../../` in JS)
>     }
> }
> ```

---

## 📥 `use` (The JS `import`)

Bring items into scope to avoid long paths.

```rust
use std::collections::HashMap;
use std::io::Result as IoResult; // Aliasing (like `import { x as y }`)

fn main() {
    let mut map = HashMap::new();
}
```

---

## 💡 Takeaway for Frontend
Rust's `mod` is like your folder structure, and `pub` is like `export`. The biggest change: in JS, everything in a file is "visible" to the file. In Rust, you have to explicitly `pub` modules AND the items inside them to make them accessible from the outside. Use **`use`** to pull that code in just like an **`import`**.