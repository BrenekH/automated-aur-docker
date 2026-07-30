FROM docker.io/lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin update-package-via-manifest

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --bin update-package-via-manifest
COPY . .
RUN cargo build --release --bin update-package-via-manifest

FROM archlinux/archlinux:base-devel

RUN pacman -Syu --noconfirm --needed sudo git pacman-contrib &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers

# Delay copying index.js from the build stage because buildx (which is used when building on GH Actions) can parallelize the different stages.
COPY --from=builder /app/target/release/update-package-via-manifest /update-package-via-manifest

USER builder

ENTRYPOINT [ "/update-package-via-manifest" ]
