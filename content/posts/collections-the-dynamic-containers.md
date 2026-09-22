---
title: "Collections: The \"Dynamic Containers\""
date: 2026-04-07
slug: collections-the-dynamic-containers
description: "In JavaScript, every Array or Object is dynamic. You can add or remove elements whenever you want. In Rust, we use Collections when we need that same ..."
tags: [rust, list, collections, hashmap, vec]
---



In JavaScript, every Array or Object is dynamic. You can add or remove elements whenever you want. In Rust, we use **Collections** when we need that same flexibility. 

Think of Collections as **Dynamic Toolboxes**. While Primitives like arrays have a fixed size, Collections grow and shrink as your application needs more space, and they are always stored on the **Heap**.

---

## 🏗️ The 3-Step Narrative

### 1. `Vec<T>`: The "Growable List"
A **Vector** is essentially a JS Array. It stores a list of values of the **same type** right next to each other in memory.
* **What it unlocks:** The ability to push, pop, and iterate over items without knowing the final count at compile-time.

```rust
let mut v = Vec::new(); // 📦 Creating an empty vector
v.push(1); // 🪄 Adding items
v.push(2);

let v2 = vec![1, 2, 3]; // ✨ Using the macro for instant creation
```

### 2. `HashMap<K, V>`: The "Dictionary"
A **HashMap** is like a JS Object or a `Map`. It stores key-value pairs where every key is unique. 
* **What it unlocks:** Quick lookup of data based on a key (e.g., finding a "User" by their "ID").

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert(String::from("Blue"), 10); // 🏷️ Inserting a record
scores.insert(String::from("Yellow"), 50);
```

### 3. The Entry Contract
Rust's HashMap has a unique "Entry API" that makes updating data feel like a clean contract.

```rust
let mut counts = HashMap::new();
let text = "hello world";

for word in text.split_whitespace() {
    // 🪄 "or_insert" only inserts if the key doesn't exist
    let count = counts.entry(word).or_insert(0);
    *count += 1;
}
```

---

## 🔬 From the Official Book

### ⚡ Iterating Over Collections
You spend most of your time iterating. Use **`.iter()`** to borrow the data, or **`.into_iter()`** to take ownership of it.

```rust
let v = vec![100, 32, 57];
for i in &v {
    println!("{i}"); // 🔍 Borrowing
}
```

---

## 💡 What Collections Unlock
- **Real-World Data**: You can't always know how many items a user will input. `Vec` and `HashMap` allow your application to scale with its data.
- **Efficient Filtering**: Combined with ****Iterators and Closures****, you can transform your collections with the same speed as a manual `for` loop.
- **Safety by Default**: Rust ensures that you never "go out of bounds" in a way that crashes your program—it will return an `Option::None` instead!