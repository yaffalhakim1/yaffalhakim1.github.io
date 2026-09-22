---
title: "How to Create a Performant \"Listening\" Table of Contents in React"
date: 2026-01-11
slug: how-to-create-a-performant-listening-table-of-contents-in-react
description: "Building a Table of Contents (TOC) that \"listens\" to your scroll position is a great way to improve UX, especially for long-form content like Terms an..."
tags: [react-javascript-webdev-frontend-ux]
---



Building a Table of Contents (TOC) that "listens" to your scroll position is a great way to improve UX, especially for long-form content like Terms and Conditions or Documentation. In this guide, we’ll build a TOC that highlights the active section as you scroll, with a focus on performance.

### **1\. Define Your Data Structure**

First, we need an array of sections. We use `Object.freeze` to ensure this metadata remains constant and doesn't get re-declared on every render.

JavaScript

```javascript
const SECTIONS = Object.freeze([
  { id: 'umum', title: 'Informasi Umum' },
  { id: 'penggunaan-dan-penghentian', title: 'Penggunaan dan penghentian layanan' },
  { id: 'pernyataan-dan-persetujuan', title: 'Pernyataan dan Persetujuan' },
  { id: 'larangan-dan-tanggung-jawab', title: 'Larangan dan Tanggung Jawab Pengguna' },
  { id: 'pembatasan-tanggung-jawab', title: 'Pembatasan Tanggung Jawab' },
  { id: 'hubungi-kami', title: 'Hubungi Experience' }
]);
```

### **2\. The Implementation Logic**

The strategy is simple:

1. Listen to the `scroll` event.
    
2. Check the `offsetTop` of each section.
    
3. Compare it against the current scroll position + an offset (to account for your Header/Navbar).
    

### **3\. The Optimized Code**

We will add **Debouncing** to our scroll listener. Without this, the function runs hundreds of times per second, which can cause lag on mobile devices.

JavaScript

```javascript
import { useState, useEffect } from 'react';

export default function TermsAndConditionsPage() {
  const [activeSection, setActiveSection] = useState('');
  const HEADER_OFFSET = 150; // Height of your sticky header in pixels

  // Helper function to scroll to a section when clicking the TOC
  const scrollToSection = (id) => {
    const element = document.getElementById(id);
    if (element) {
      const top = element.getBoundingClientRect().top + window.scrollY - HEADER_OFFSET;
      window.scrollTo({ top, behavior: 'smooth' });
    }
  };

  const updateActiveSection = () => {
    const scrollPosition = window.scrollY;
    
    // 1. Check if we are at the very bottom of the page
    if (window.innerHeight + scrollPosition >= document.body.offsetHeight - 10) {
      setActiveSection(SECTIONS[SECTIONS.length - 1].id);
      return;
    }

    // 2. Iterate backwards to find the current active section
    for (let i = SECTIONS.length - 1; i >= 0; i--) {
      const section = document.getElementById(SECTIONS[i].id);
      if (section && section.offsetTop <= scrollPosition + HEADER_OFFSET + 10) {
        setActiveSection(SECTIONS[i].id);
        break;
      }
    }
  };

  useEffect(() => {
    let timeoutId = null;

    const debouncedScroll = () => {
      if (timeoutId) return;
      timeoutId = setTimeout(() => {
        updateActiveSection();
        timeoutId = null;
      }, 50); // Run logic only once every 50ms
    };

    window.addEventListener('scroll', debouncedScroll);
    updateActiveSection(); // Initial check

    return () => {
      window.removeEventListener('scroll', debouncedScroll);
      if (timeoutId) clearTimeout(timeoutId);
    };
  }, []);

  return (
    <div style={{ display: 'flex' }}>
      {/* Sidebar Table of Contents */}
      <nav style={{ position: 'sticky', top: '150px', height: 'fit-content' }}>
        {SECTIONS.map((section) => (
          <div
            key={section.id}
            onClick={() => scrollToSection(section.id)}
            style={{
              cursor: 'pointer',
              fontWeight: activeSection === section.id ? 'bold' : 'normal',
              color: activeSection === section.id ? '#3182CE' : 'black',
              borderLeft: activeSection === section.id ? '3px solid #3182CE' : '3px solid transparent',
              paddingLeft: '12px',
              transition: 'all 0.2s ease',
              marginBottom: '10px'
            }}
          >
            {section.title}
          </div>
        ))}
      </nav>

      {/* Main Content */}
      <main>
        {SECTIONS.map((section) => (
          <section id={section.id} key={section.id} style={{ minHeight: '600px' }}>
            <h1>{section.title}</h1>
            <p>Your content goes here...</p>
          </section>
        ))}
      </main>
    </div>
  );
}
```

---

### **Why these fixes matter:**

1. **Debouncing (50ms):** This reduces the CPU load significantly. The user won't notice a 50ms delay, but their battery and browser performance will thank you.
    
2. **The Bottom Check:** Often, the last section isn't long enough to reach the top of the screen. Without `document.body.offsetHeight`, the last item in your TOC might never get highlighted.
    
3. **Reverse Loop:** By starting from the bottom of the array (`SECTIONS.length - 1`), the code finds the *most recent* section you’ve passed.
    
4. **BoundingClientRect for Smooth Scroll:** Using this ensures that when you click a link, it lands exactly where you expect, regardless of how many relative elements are on the page.
    

### **Summary**

Adding a listening indicator is 50% logic and 50% performance. By combining `offsetTop` with a debounced scroll listener, you get a smooth, professional-feeling navigation that works across all devices.