FROM archlinux/archlinux:base-devel

COPY buildpkg.py entrypoint.sh /

RUN pacman -Syu --noconfirm --needed python sudo git namcap &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
	mkdir /buildout && chown -R builder:builder /buildout &&\
	chmod +x /buildpkg.py /entrypoint.sh

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
