FROM docker.io/lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin publish-package

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --bin publish-package
COPY . .
RUN cargo build --release --bin publish-package

FROM archlinux/archlinux:base-devel

COPY publish-package/entrypoint.sh /

RUN pacman -Syu --noconfirm --needed sudo git openssh &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
	chmod +x /publishpkg.py /entrypoint.sh

COPY --from=builder /app/target/release/publish-package /publish-package

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
