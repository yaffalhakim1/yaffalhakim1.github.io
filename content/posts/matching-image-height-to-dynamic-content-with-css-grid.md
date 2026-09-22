---
title: "Matching Image Height to Dynamic Content with CSS Grid"
date: 2026-01-11
slug: matching-image-height-to-dynamic-content-with-css-grid
description: "Have you ever built a split-screen layout where one side is a long form, and the other is an image? Often, the image either fails to fill the vertical..."
tags: [ux, frontend-development, nextjs, chakra-ui]
---



Have you ever built a split-screen layout where one side is a long form, and the other is an image? Often, the image either fails to fill the vertical space or creates unwanted white space.

The "Magic" solution involves a combination of **CSS Grid's Stretch alignment** and **Absolute Positioning**.

### **The Problem**

By default, an image will try to maintain its aspect ratio. If your content (like a form) is taller than your image, you get a gap. If you force the image to `height: 100%`, it might stretch unless you use `object-fit` but getting it to respect the parent's grid height can still be finicky.

### **The Solution: The "Absolute Fill" Technique**

The strategy is to:

1. Make the **Grid Container** stretch its items, so they are both equal in height.
    
2. Set the **Grid Item** (the image container) to `position: relative`.
    
3. "Collapse" the image's own influence on height by making it `position absolute`, then forcing it to fill the parent's dimensions.
    

### **Implementation (React + Chakra UI)**

JavaScript

```javascript
<Grid 
  templateColumns={{ base: "1fr", lg: "1fr 1fr" }} 
  alignItems="stretch" // CRITICAL: This ensures both GridItems are the same height
>
  {/* Image Container */}
  <GridItem 
    position="relative" 
    display={{ base: "none", lg: "block" }} // Example: Hide on mobile or handle differently
  >
    <NextImage
      src={imageSrcs}
      alt="Feature Screenshot"
      fill // Use 'fill' for Next.js 13+ or the properties below
      style={{ objectFit: "contain", objectPosition: "bottom" }}
      
      /* Manual properties if not using 'fill' */
      position="absolute"
      inset={0}
      width="100%"
      height="100%"
    />
  </GridItem>

  {/* Content Container (The "Master" height) */}
  <GridItem p={{ base: 4, lg: 8 }}>
    <VStack align="start" spacing={6}>
      <Text fontSize="lg">Please fill out the form below correctly.</Text>
      
      {/* Your Form Logic Here */}
      <Box width="100%">
        <RHFFormProvider {...methods}>
           {/* Form Inputs... */}
        </VStack>
      </Box>
    </VStack>
  </GridItem>
</Grid>
```

### **Why this works:**

* `alignItems="stretch"`: This is the secret sauce. It tells the Grid that all children should match the height of the tallest child (usually the form).
    
* `position="relative"` on GridItem: This creates a boundary. Even though the GridItem has height (inherited from the form), it doesn't have a "fixed" height.
    
* `position="absolute"` & `inset={0}` on Image: By taking the image out of the document flow, it no longer tries to "push" the container's height. Instead, it looks at the `relative` parent and says, "I will be exactly as big as you are."
    
* `object-fit: contain/cover`: This ensures that as the form height changes, the image scales gracefully without distorting.
    

### **Pro-Tip for Next.js Users**

If you are using `next/image`, using the `fill` prop combined with a parent that has `position: relative` is the modern way to achieve this. It automatically applies the absolute positioning logic for you!