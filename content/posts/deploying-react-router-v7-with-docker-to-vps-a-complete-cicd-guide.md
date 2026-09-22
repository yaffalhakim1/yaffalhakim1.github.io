---
title: "Deploying React Router v7 with Docker to VPS: A Complete CI/CD Guide"
date: 2026-01-11
slug: deploying-react-router-v7-with-docker-to-vps-a-complete-cicd-guide
description: "--- Introduction Deploying a modern React Router v7 application to production requires more than just copying files to a server. In this comprehens..."
tags: [deployment, frontend-development, cicd]
---



---

## Introduction

Deploying a modern React Router v7 application to production requires more than just copying files to a server. In this comprehensive guide, I'll walk you through the complete deployment setup I use for my portfolio, including Docker containerization, GitHub Actions CI/CD, and a Caddy reverse proxy on a VPS.

This guide uses **React Router v7** with SSR (Server-Side Rendering), **Docker** for containerization, **GitHub Actions** for CI/CD, and **Caddy** as a reverse proxy with automatic HTTPS.

### Why VPS instead of Vercel?

While platforms like Vercel or Netlify are excellent for getting started, I chose a VPS (Virtual Private Server) for this project to demonstrate **infrastructure ownership**. Mastering VPS deployment gives you:

1. **Full Control**: No vendor lock-in or platform-specific limitations.
    
2. **Cost Efficiency**: A fixed cost (e.g., $5/mo) regardless of bandwidth spikes or build minutes.
    
3. **System Engineering Skills**: Managing your own Docker containers, reverse proxies, and firewalls is a critical skill for senior engineers.
    

## Architecture Overview

Here's how our deployment pipeline works:

1. **Local Development**: Write code and test locally using `npm run dev`.
    
2. **GitHub Actions Trigger**: Manually trigger the deployment workflow from GitHub Actions.
    
3. **Docker Build & Push**: Build the Docker image and push it to Docker Hub.
    
4. **VPS Deployment**: The workflow SSHs into the VPS, pulls the latest image, and restarts containers.
    
5. **Caddy Proxy**: Caddy serves the app with automatic HTTPS and security headers.
    

## Project Structure

First, let's look at the relevant files in our project:

* `Dockerfile`
    
* `docker-compose.yml`
    
* `Caddyfile`
    
* `vite.config.docker.ts`
    
* `.github/workflows/deploy.yaml`
    
* `dockerignore`
    

## Step 1: Dockerfile Configuration

Our Dockerfile uses a multi-stage build to optimize the final image size. Here's the complete setup:

```dockerfile
# Stage 1: Build stage
FROM node:20-alpine AS builder

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies (clean install)
RUN npm ci

# Copy application files
COPY . .

# Build application using Docker-specific Vite config
RUN npm run build -- --config vite.config.docker.ts

# Prune dev dependencies to keep the image small
RUN npm prune --production

# Stage 2: Production stage
FROM node:20-slim AS runner
WORKDIR /app

# COPY the necessary files
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/node_modules ./node_modules

# --- FIX START ---
# Set the environment to production
ENV NODE_ENV=production

# Use the built-in non-root user for security
USER node

# Ensure the app listens on all interfaces (0.0.0.0)
ENV PORT=3000
EXPOSE 3000
# --- FIX END ---

CMD ["npm", "run", "start"]
```

> 💡 **Tip**: The multi-stage build pattern reduces the final image size by excluding build tools and dev dependencies from the production image. My final image dropped from ~900MB to ~120MB using this method.

## Step 2: Docker Compose Configuration

The `docker-compose.yml` defines our production services:

```yaml
services:
  portfolio-yafi:
    image: yaffalhakim1/portfolio-yafi:latest
    container_name: portfolio-yafi
    restart: unless-stopped
    ports:
      - '3000:3000'
    environment:
      - NODE_ENV=production
    networks:
      - web
    healthcheck:
      test:
        [
          'CMD',
          'wget',
          '--quiet',
          '--tries=1',
          '--spider',
          'http://localhost:3000/ health',
        ]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '1'

  caddy:
    image: caddy:2-alpine
    container_name: caddy
    restart: unless-stopped
    ports:
      - '80:80'
      - '443:443'
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config
    networks:
      - web
    depends_on:
      - portfolio-yafi

networks:
  web:
    image: your-username/your-repo:latest
    ports:
      - "3000:3000"
    restart: always
    # --- FIX START ---
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

volumes:
  caddy_data:
  caddy_config:
```

### Configuration Highlights:

* **Resource Limits**: 512MB memory limit prevents a single container from crashing the entire server.
    
* **Health Checks**: Automatically restarts the container if the app becomes unresponsive.
    
* **Bridge Network**: Isolates our services from the outside world; only Caddy exposes ports 80/443.
    

## Step 3: Vite Docker Configuration

Create a specialized Vite config for Docker builds (`vite.config.docker.ts`):

```typescript
import { reactRouter } from '@react-router/dev/vite';
import { defineConfig } from 'vite';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [reactRouter(), tsconfigPaths()],
  ssr: {
    // Crucial for packages that don't support SSR native ESM
    noExternal: ['react-slick', 'cobe', '@react-spring/web'],
  },
  build: {
    target: 'esnext',
  },
});
```

## Step 4: Caddy Reverse Proxy

Caddy provides automatic HTTPS and acts as a reverse proxy. Here's the `Caddyfile`:

```plaintext
:80 {
    reverse_proxy portfolio-yafi:3000

    # Security headers
    header {
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        X-XSS-Protection "1; mode=block"
        Referrer-Policy "strict-origin-when-cross-origin"
    }

    # Enable compression
    encode gzip

    # Logging
    log {
        output file /var/log/caddy/access.log
        format json
    }
}
```

## Step 5: GitHub Actions CI/CD

The `.github/workflows/deploy.yaml` automates our deployment:

```yaml
name: Deploy to VPS

on:
  workflow_dispatch:

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      # --- FIX START ---
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: your-username/your-repo:latest
          cache-from: type=gha # Use GitHub Actions cache
          cache-to: type=gha,mode=max

      - name: Deploy to VPS
        uses: appleboy/ssh-action@master
        with:
          host: ${{ secrets.VPS_HOST }}
          username: ${{ secrets.VPS_USERNAME }}
          key: ${{ secrets.VPS_SSH_KEY }}
          script: |
            cd /opt/portfolio
            docker compose pull
            docker compose up -d
            docker system prune -f
```

### Required GitHub Secrets:

Create these Docker Hub related secrets:

* `DOCKER_USERNAME`: Your Docker Hub username
    
* `DOCKER_PASSWORD`: Your Docker Hub Access Token (Security Tip: Do NOT use your real password. Generate a token!)
    

Create these SSH related secrets:

* `VPS_HOST`: Server IP address
    
* `VPS_USERNAME`: User (e.g., 'root' or 'ubuntu')
    
* `VPS_SSH_KEY`: Private SSH Key content
    

## Step 6: VPS Setup

On your VPS, prepare the environment:

```bash
# SSH into your VPS
ssh user@your-vps-ip

# Create deployment directory
mkdir -p /opt/portfolio
cd /opt/portfolio

# Create files (or scp them)
touch docker-compose.yml Caddyfile
```

## Security & Maintenance

> ⚠️ **Security Warning**:
> 
> * **SSH Keys**: Ensure your VPS only accepts SSH key authentication (disable password login).
>     
> * **Firewall**: Use `ufw` to allow only ports 22, 80, and 443.
>     
> * **Secrets**: Never commit `.env` files or keys to GitHub.
>     

<div data-node-type="callout">
<div data-node-type="callout-emoji">💡</div>
<div data-node-type="callout-text"><strong>⚠️ Important Note on Environment Variables:</strong> In React Router v7, variables prefixed with <code>VITE_</code> are baked into the code <strong>during the build step</strong> (on GitHub Actions). Variables used by the server (like <code>DATABASE_URL</code>) are read <strong>at runtime</strong> (on your VPS). If your client-side code needs a <code>VITE_API_URL</code>, you must include it in your GitHub Secrets and pass it as an <code>--build-arg</code> in your Dockerfile, otherwise it will be <code>undefined</code> in production.</div>
</div>

## Conclusion

This setup provides a robust, automated deployment pipeline. By combining **Docker** for consistency, **GitHub Actions** for automation, and **Caddy** for security, we've built a production-grade infrastructure that rivals managed platforms, but with a fraction of the cost and significantly more control. The best part? Once configured, deploying is as simple as clicking "Run workflow" in GitHub Actions!

## Related Resources

Official documentation for React Router v7 Learn more about containerization Automatic HTTPS reverse proxy

---

Have questions or suggestions? Feel free to reach out!