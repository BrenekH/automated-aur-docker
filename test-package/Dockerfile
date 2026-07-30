FROM docker.io/lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin test-package

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --bin test-package
COPY . .
RUN cargo build --release --bin test-package

FROM archlinux/archlinux:base

COPY entrypoint.sh /

RUN pacman -Syu --noconfirm &&\
 chmod +x /entrypoint.sh

COPY --from=builder /app/target/release/test-package /test-package

ENTRYPOINT [ "/entrypoint.sh" ]
