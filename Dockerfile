FROM rust:latest as rust-builder

WORKDIR /server

RUN cargo init --name file-downloader

COPY server/Cargo.toml server/Cargo.lock ./

RUN cargo build --release

COPY server/ .

RUN touch ./src/main.rs

RUN cargo build --release

FROM node:lts as node-builder

WORKDIR /front

COPY front/package.json front/package-lock.json ./

RUN npm ci

COPY front/ .

RUN npm run build

FROM debian:bullseye-slim

RUN apt update -y && apt upgrade -y
RUN apt install openssl ca-certificates curl -y

COPY --from=rust-builder /server/target/release/file-downloader /server/file-downloader
COPY --from=node-builder /front/dist ./front/dist

RUN mkdir config
RUN mkdir downloads

HEALTHCHECK CMD curl --fail http://localhost:8055/health || exit 1

EXPOSE 8055

ENTRYPOINT ["/server/file-downloader"]
