---
title: "Engineering a Better Auto Perfect Guard for Sword & Shield in Monster Hunter Wilds"
date: 2026-02-05
slug: engineering-a-better-auto-perfect-guard-for-sword-and-shield-in-monster-hunter-wilds
description: "Learning from Existing Mods and Building a Better Auto Perfect Guard for SnS When I decided to build an Auto Perfect Guard for Sword & Shield (SnS) i..."
tags: [lua, mods, monster-hunter-wilds]
---



### Learning from Existing Mods and Building a Better Auto Perfect Guard for SnS

When I decided to build an Auto Perfect Guard for Sword & Shield (SnS) in *Monster Hunter Wilds*, I didn’t start by writing code. I started by reading other people’s code.

The modding community already had working solutions, and it didn’t make sense to ignore that. Reverse-engineering real, battle-tested mods teaches you how the game actually behaves much faster than guessing.

I spent time studying two existing mods:

- [Auto Perfect Guard for Lance/SnS and Counter for Switch Axe](https://www.nexusmods.com/monsterhunterwilds/mods/738?tab=description)  
- [Auto Counter for Switch Axe](https://www.nexusmods.com/monsterhunterwilds/mods/2579?tab=description)

I wasn’t trying to copy them. I wanted to understand how they detect hits, when they attempt guard injections, and how they deal with failure. That last part turned out to be more important than I expected.

Both mods were good. They clearly worked in real hunts. But neither was fully tailored to SnS. Sword & Shield has fast transitions, frequent animation locks, and guard states that don’t behave like heavier weapons. I wanted something designed specifically around those realities.

---

### The First Version

My first implementation followed a simple idea: detect a hit, open a small timing window, and inject a guard. On paper, that’s exactly how a human would try to perfect guard.

And it worked. At least at first.

In controlled situations, it felt reliable. But actual hunts are messy. Monsters use delayed explosions, multi-hit attacks, and strange angles. Some guards didn’t trigger. Others triggered late. It wasn’t broken, just inconsistent.

Then I noticed something more concerning: occasional FPS drops.

---

### The Lag That Didn’t Make Sense

The lag wasn’t constant. It only happened in certain situations, especially when I got hit during long animations or while super armor was active. That detail mattered. If the script were simply heavy, performance would drop all the time.

So this wasn’t about computation cost. It was about behavior.

I started observing how often my guard injection ran. That’s when the pattern showed up. When guards succeeded, everything stayed normal. When they failed, the number of attempts spiked.

The system wasn’t just retrying. It was retrying every frame.

---

### The Real Bug

The core logic looked like this:

```lua
if not guardStart then
    guardStart = now
end

if (now - guardStart) <= window then
    tryGuard()
end
```

At a glance, it’s reasonable. But there’s a hidden assumption: that retrying is harmless.

What actually happens in Monster Hunter is that the engine sometimes refuses a guard if you’re in certain animations. It doesn’t throw an error. It doesn’t notify you. It just ignores the request.

So when `tryGuard()` failed, my logic didn’t see it as a signal. It just tried again next frame. And then again. At high FPS, one hit could cause dozens or even hundreds of attempts.

It wasn’t an infinite loop in code, but it behaved like one in practice. The engine was being spammed with requests it had already decided to reject.

That’s what caused the FPS drops.

---

### Rethinking the Approach

The solution wasn’t to make detection smarter or faster. The solution was to accept that the engine can say no, and to treat that as real feedback.

I added a small debounce between attempts so the script would wait before retrying.

```lua
local cooldown = 0.05

if now - lastAttempt >= cooldown then
    if tryGuard() then
        lastAttempt = now
        handled = true
    end
end
```

With this change, even if the engine refused the action, the script wouldn’t hammer it every frame. It would pause, then try again later.

That small change completely removed the FPS drops.

---

### Improving Detection

After stabilizing performance, I revisited detection. One thing I learned from Switch Axe mods is that relying on a single detection point is fragile. Some attacks simply don’t pass through the checks you expect.

So I expanded detection across multiple stages of the hit validation pipeline. Instead of trusting one hook, the system observes more than one phase.

This didn’t make the logic more complex, just more aware. AOE attacks, off-angle hits, and multi-hit scenarios became far more consistent.

---

### What I Took Away From This

This project taught me that auto-guard systems aren’t just about timing. They’re about understanding how the engine manages state and when it allows or refuses actions.

The biggest lesson was that failure handling matters as much as success handling. If the engine rejects an action, that’s information, not noise.

In the end, I didn’t make the guard faster. I made it more respectful of the engine. And once the script stopped fighting the game’s rules, everything became more stable.

Sometimes reliability doesn’t come from doing more. It comes from knowing when to wait.