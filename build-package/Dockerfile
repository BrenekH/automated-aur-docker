FROM archlinux/archlinux:base-devel

COPY buildpkg.py entrypoint.sh /

RUN pacman -Syu --noconfirm --needed python sudo git &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
	mkdir /buildout && chown -R builder:builder /buildout &&\
	chmod +x /buildpkg.py /entrypoint.sh

# Install namcap-git from AUR instead of namcap from repos
# Fixes https://github.com/BrenekH/automated-aur-docker/issues/3
COPY namcap-git-3.2.10.r10.ga6f9760-1-any.pkg.tar.zst /tmp
RUN pacman -U --noconfirm /tmp/namcap-git-3.2.10.r10.ga6f9760-1-any.pkg.tar.zst && rm /tmp/namcap-git-3.2.10.r10.ga6f9760-1-any.pkg.tar.zst

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
