# Stage 1: Build WASM
FROM rust:1.78 AS wasm-builder
RUN cargo install wasm-pack
WORKDIR /app
COPY crates/motarjim-wasm/ Cargo.toml Cargo.lock ./
RUN wasm-pack build --target web

# Stage 2: Build web app
FROM node:20 AS web-builder
WORKDIR /app
COPY apps/web/package.json apps/web/package-lock.json ./
RUN npm ci
COPY apps/web/ .
COPY --from=wasm-builder /app/pkg ./public/wasm
RUN npm run build

# Stage 3: Serve
FROM nginx:alpine
COPY --from=web-builder /app/dist /usr/share/nginx/html
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
