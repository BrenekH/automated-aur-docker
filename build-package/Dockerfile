FROM archlinux/archlinux:base-devel

COPY buildpkg.py entrypoint.sh /
COPY custom-paru /opt/paru

RUN pacman -Syu --noconfirm --needed python sudo git namcap &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
	mkdir /buildout && chown -R builder:builder /buildout &&\
	chmod +x /buildpkg.py /entrypoint.sh /opt/paru

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
