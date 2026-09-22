---
title: "Building a Universal \"Scroll-To\" Hook in React & Next.js"
date: 2026-01-11
slug: building-a-universal-scroll-to-hook-in-react-and-nextjs
description: "On a landing page, navigation usually involves smooth scrolling to sections. However, once you add multiple pages (like a Blog or FAQ), a simple eleme..."
tags: [ux, reactjs, nextjs]
---



On a landing page, navigation usually involves smooth scrolling to sections. However, once you add multiple pages (like a Blog or FAQ), a simple `element.scrollIntoView()` isn't enough. You need a way to navigate back to the home page *and then* scroll to the section.

### **The Math Behind the Scroll**

To get the exact position for `window.scrollTo`, we use a specific formula. Think of it like finding a specific page in a book:

1. `getBoundingClientRect().top`: How far the element is from the *current* top of your screen.
    
2. `window.scrollY`: How far you have already scrolled down the page.
    
3. `offset`: Space for your sticky header so it doesn't cover the section title.
    

**The Formula:** `Target = Viewport Top + Current Scroll - Offset`

---

### **The Implementation:** `useScrollTo`

This hook handles three scenarios:

1. **External Links**: Redirects normally.
    
2. **Same-page Scroll**: Scrolls smoothly and updates the URL hash.
    
3. **Cross-page Scroll**: Navigates to the correct page first, then scrolls after the page loads.
    

JavaScript

```javascript
import { useRouter } from "next/router";

/**
 * A universal hook for smooth scrolling that handles 
 * both same-page and cross-page navigation.
 */
export function useScrollTo() {
  const router = useRouter();

  const scrollTo = (href, offset = 0) => {
    if (typeof window === "undefined") return;

    // 1. Handle External Links
    if (href.startsWith("http")) {
      window.location.href = href;
      return;
    }

    const targetId = href.startsWith("#") ? href.substring(1) : href;
    const targetElement = document.getElementById(targetId);

    // 2. Scenario: Element exists on current page
    if (targetElement) {
      const targetPosition =
        targetElement.getBoundingClientRect().top + window.scrollY - offset;
      
      window.scrollTo({ top: targetPosition, behavior: "smooth" });
      
      // Update URL hash without a full page refresh
      window.history.pushState(null, "", `#${targetId}`);
    } 
    // 3. Scenario: Element is on a different page
    else {
      // Navigate to the root (or specific page) with the hash
      router.push(`/#${targetId}`).then(() => {
        // Wait briefly for the new page to mount
        setTimeout(() => {
          const newTarget = document.getElementById(targetId);
          if (newTarget) {
            const newPos = newTarget.getBoundingClientRect().top + window.scrollY - offset;
            window.scrollTo({ top: newPos, behavior: "smooth" });
          }
        }, 150);
      });
    }
  };

  return scrollTo;
}
```

---

### **How to Use It**

Simply call the hook in your component and pass the ID of the section you want to reach.

JavaScript

```javascript
const scrollTo = useScrollTo();

<Button onClick={() => scrollTo("pricing-section", 80)}>
  View Pricing
</Button>
```

### **Why This Approach is Better:**

* **URL Synchronicity:** It uses `window.history.pushState`. If the user refreshes or shares the link, the `#id` stays in the URL, allowing the browser to find the spot again.
    
* **The "Header Gap" Fix:** Most landing pages have a sticky navbar. By passing an `offset`, you prevent the navbar from overlapping your section headings.
    
* **Next.js Integration:** Using `router.push().then()` ensures that we only attempt to scroll *after* Next.js has finished the route transition.
    

---

### **Pro-Tip: Why the** `setTimeout`?

In the cross-page scenario, we use a small delay (100ms–150ms). This is because Next.js needs a split second to render the DOM elements on the new page before `document.getElementById` can actually find them.