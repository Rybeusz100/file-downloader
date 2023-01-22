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

FROM debian:buster-slim

RUN apt update && apt upgrade -y
RUN apt install openssl sqlite ca-certificates -y

COPY --from=rust-builder /server/target/release/file-downloader .
COPY --from=node-builder /front/dist ./front

RUN mkdir config
RUN mkdir downloads

ENTRYPOINT ["./file-downloader"]

EXPOSE 8055
