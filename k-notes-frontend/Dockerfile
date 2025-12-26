# Build stage
FROM oven/bun:1 AS builder

WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY . .
RUN bun run build

# Production stage
FROM nginx:alpine

# Copy built assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy custom nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Create script to generate env-config.js from environment variables
RUN echo '#!/bin/sh' > /docker-entrypoint.d/40-env-config.sh && \
    echo 'cat > /usr/share/nginx/html/env-config.js << "EOF"' >> /docker-entrypoint.d/40-env-config.sh && \
    echo '// Runtime environment configuration' >> /docker-entrypoint.d/40-env-config.sh && \
    echo '// Generated at container startup' >> /docker-entrypoint.d/40-env-config.sh && \
    echo 'window.env = {' >> /docker-entrypoint.d/40-env-config.sh && \
    echo 'EOF' >> /docker-entrypoint.d/40-env-config.sh && \
    echo 'if [ -n "$API_URL" ]; then' >> /docker-entrypoint.d/40-env-config.sh && \
    echo '  echo "  API_URL: \"$API_URL\"," >> /usr/share/nginx/html/env-config.js' >> /docker-entrypoint.d/40-env-config.sh && \
    echo 'fi' >> /docker-entrypoint.d/40-env-config.sh && \
    echo 'echo "};" >> /usr/share/nginx/html/env-config.js' >> /docker-entrypoint.d/40-env-config.sh && \
    chmod +x /docker-entrypoint.d/40-env-config.sh

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
