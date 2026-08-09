FROM docker.io/archlinux/archlinux:base-devel AS chef
RUN pacman -Syu --noconfirm --needed rust cargo clang &&\
	cargo install --locked cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin build-package

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --bin build-package
COPY . .
RUN cargo build --release --bin build-package

FROM archlinux/archlinux:base-devel

COPY build-package/entrypoint.sh /

RUN pacman -Syu --noconfirm --needed sudo git namcap &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
	mkdir /buildout && chown -R builder:builder /buildout &&\
	chmod +x /entrypoint.sh &&\
	echo "MAKEFLAGS=\"-j\$(nproc)\"" >> /etc/makepkg.conf

COPY --from=builder /app/target/release/build-package /build-package

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
