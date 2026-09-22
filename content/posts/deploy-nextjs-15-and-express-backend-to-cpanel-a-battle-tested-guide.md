---
title: "Deploy Next.js 15 & Express Backend to cPanel: A Battle-Tested Guide"
date: 2026-01-27
slug: deploy-nextjs-15-and-express-backend-to-cpanel-a-battle-tested-guide
description: "> What I learned from a frustrating but educational deployment journey Deploying modern JavaScript frameworks to traditional cPanel hosting can feel ..."
tags: [deployment, nextjs, cpanel]
---



> What I learned from a frustrating but educational deployment journey

Deploying modern JavaScript frameworks to traditional cPanel hosting can feel like fitting a square peg into a round hole. After hours of debugging `EAGAIN` errors, CORS issues, and resource limits, I finally got my Next.js 15 frontend and Express.js backend running smoothly on Jagoan Hosting's cPanel. Here's everything I learned.

## The Stack

- **Frontend**: Next.js 15, React 19, Material-UI v7
- **Backend**: Express.js, TypeScript, Sequelize
- **Hosting**: Jagoan Hosting Cloud NextGen (cPanel with Node.js support)
- **Domains**:
  - Frontend: `dapurenakeeco.com`
  - Backend API: `api.dapurenakeeco.com`

---

## Part 1: Deploying the Express Backend

The backend was relatively straightforward. Here's the process:

### Step 1: Build Locally

```bash
cd restaurant-backend
npm run build
```

This compiles TypeScript to JavaScript in the `dist/` folder.

### Step 2: Prepare Files for Upload

Create a zip containing:

- `dist/` (compiled code)
- `package.json`
- `package-lock.json`
- `.env` (with production values)

**DO NOT include `node_modules`** - we'll install on the server.

### Step 3: Setup Node.js App in cPanel

1. Go to **cPanel → Setup Node.js App**
2. Click **Create Application**
3. Configure:
   - **Node.js Version**: 20.x
   - **Application Mode**: Production
   - **Application Root**: `restaurant-backend`
   - **Application URL**: `api.dapurenakeeco.com`
   - **Startup File**: `dist/app.js`

### Step 4: Upload & Install

1. Upload your zip to the `restaurant-backend` folder
2. Extract it
3. Click **Run NPM Install** in the Node.js App interface
4. Click **Start App**

### CORS Configuration (Critical!)

Make sure your backend allows requests from your frontend domain:

```typescript
// app.ts
app.use(
  cors({
    origin: [
      "https://dapurenakeeco.com",
      "http://dapurenakeeco.com",
      "http://localhost:3000",
    ],
    credentials: true,
    methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
  }),
);
```

**Pro tip**: Double-check your domain spelling! I wasted time because of a typo (`dapurenakeco` vs `dapurenakeeco`).

---

## Part 2: Deploying Next.js 15 (The Hard Part)

This is where things got interesting. Modern Next.js is resource-hungry, and shared hosting has strict limits.

### The Problem: `EAGAIN` Errors

When I tried `npm install` on the server, I got:

```
Error: spawn npm EAGAIN
errno: -11
code: 'EAGAIN'
syscall: 'spawn npm'
```

**Translation**: The server hit its process limit (nProc). Shared hosting typically limits you to 20-50 concurrent processes, and `npm install` for a Next.js project spawns way more than that.

### The Solution: Custom Server + Local Dependencies

After many failed attempts, I found a working approach from [Jagoan Hosting's own tutorial](https://www.jagoanhosting.com/tutorial/tips/cara-upload-nextjs-di-hosting-cpanel).

#### Step 1: Create a Custom Server

Create `server.js` in your project root:

```javascript
const { createServer } = require("http");
const { parse } = require("url");
const next = require("next");

const dev = process.env.NODE_ENV !== "production";
const hostname = "localhost";
const port = process.env.PORT || 8080;

const app = next({ dev, hostname, port });
const handle = app.getRequestHandler();

app.prepare().then(() => {
  createServer(async (req, res) => {
    try {
      const parsedUrl = parse(req.url, true);
      await handle(req, res, parsedUrl);
    } catch (err) {
      console.error("Error occurred handling", req.url, err);
      res.statusCode = 500;
      res.end("internal server error");
    }
  }).listen(port, (err) => {
    if (err) throw err;
    console.log(`> Ready on http://${hostname}:${port}`);
  });
});
```

#### Step 2: Update package.json

```json
{
  "scripts": {
    "start": "NODE_ENV=production node server.js",
    "build": "next build"
  }
}
```

#### Step 3: Optimize for Shared Hosting

Add this to `next.config.js` to prevent resource exhaustion:

```javascript
const nextConfig = {
  experimental: {
    workerThreads: false,
    cpus: 1,
  },
};
```

#### Step 4: Build Locally

```bash
npm run build
```

#### Step 5: Prepare Upload Package

Zip these files/folders:

- `.next/` (build output)
- `pages/`
- `public/`
- `src/`
- `server.js`
- `package.json`
- `package-lock.json`
- `next.config.js`
- `tsconfig.json`
- `.env.local`

**DO NOT include `node_modules`** - we'll run `npm install` on the server.

---

### Step 6: Upload & Configure in cPanel

This is the most critical part. Follow these steps carefully.

#### 6.1 Create the Node.js Application

1. Log in to **cPanel**
2. Go to **Setup Node.js App**
3. Click **Create Application**
4. Fill in the settings:

| Setting                  | Value                             |
| ------------------------ | --------------------------------- |
| Node.js Version          | 20.x (or latest LTS)              |
| Application Mode         | Production                        |
| Application Root         | `e-menu-fe` (your folder name)    |
| Application URL          | `dapurenakeeco.com` (your domain) |
| Application Startup File | `server.js`                       |

5. Click **Create**

#### 6.2 Upload Your Files

1. Go to **File Manager** in cPanel
2. Navigate to `e-menu-fe` folder
3. Delete any default files (like `app.js`)
4. Click **Upload** and select your zip file
5. After upload, right-click the zip → **Extract**
6. Verify your folder structure looks like:

```
e-menu-fe/
├── .next/
├── pages/
├── public/
├── src/
├── server.js
├── package.json
├── package-lock.json
├── next.config.js
└── tsconfig.json
```

#### 6.3 Set Environment Variables

1. Go back to **Setup Node.js App**
2. Click the pencil icon (Edit) on your app
3. Scroll to **Environment Variables**
4. Click **Add Variable** and add:

| Name                | Value                         |
| ------------------- | ----------------------------- |
| PORT                | 8080                          |
| NEXT_PUBLIC_API_URL | https://api.dapurenakeeco.com |

5. Click **Save**

#### 6.4 Start the Application

1. In **Setup Node.js App**, find your app
2. Click **Run NPM Install** and wait for it to complete
3. Click **Start App** (or **Restart** if already running)
4. Wait 10-30 seconds for Next.js to initialize
5. Visit your domain to verify it's working

---

## Common Issues & Solutions

### Issue: "Couldn't find any `pages` or `app` directory"

**Cause**: Missing source files on server.

**Fix**: Make sure you uploaded `pages/`, `src/`, and `tsconfig.json`.

### Issue: "ENOENT: no such file... required-server-files.json"

**Cause**: Incomplete `.next` folder.

**Fix**: Rebuild locally with `npm run build` and re-upload the entire `.next` folder.

### Issue: "fork: Resource temporarily unavailable"

**Cause**: Too many processes running. Your account is locked.

**Fix**: Contact hosting support to kill stuck processes, or wait a few hours. Then upload with `node_modules` included to avoid server-side installs.

### Issue: CORS errors

**Cause**: Backend doesn't allow frontend origin.

**Fix**:

1. Check domain spelling in CORS config
2. Ensure both HTTP and HTTPS variants are listed
3. Rebuild and re-upload backend

### Issue: Non-standard NODE_ENV warning

**Cause**: cPanel sets a weird NODE_ENV value.

**Fix**: Set `NODE_ENV=production` explicitly in your start script (already done in our `package.json`).

---

## Key Takeaways

1. **Shared hosting has limits** - Be aware of resource limits, but modern plans can handle npm install.

2. **Use custom server.js** - The default Next.js startup doesn't play well with cPanel.

3. **cpus: 1 saves lives** - Limit worker threads to avoid hitting process limits.

4. **Double-check your domains** - Typos in CORS configs cause silent failures.

---

## Final Thoughts

Would I recommend cPanel for Next.js 15? Honestly, for small projects with limited traffic, it works. But if you're building something serious, consider:

- **Vercel** (made for Next.js)
- **Railway/Render** (easy Node.js deployment)
- **VPS with Docker** (full control)