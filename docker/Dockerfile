# Stage 1: Build WASM
FROM rust:1.85-slim-bookworm AS wasm-builder
RUN rustup target add wasm32-unknown-unknown
ENV RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\""
RUN cargo install wasm-pack
WORKDIR /build
# Copy only what's needed for wasm-pack
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN wasm-pack build crates/motarjim-wasm --target web --out-dir /wasm-pkg

# Stage 2: Build web app
FROM node:20-slim AS web-builder
WORKDIR /app
COPY apps/web/package.json apps/web/ ./
RUN npm install
COPY apps/web/ .
COPY --from=wasm-builder /wasm-pkg ./public/wasm
RUN npm run build

# Stage 3: Serve
FROM nginx:alpine
COPY --from=web-builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
