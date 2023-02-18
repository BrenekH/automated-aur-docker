FROM archlinux/archlinux:base-devel

COPY buildpkg.py entrypoint.sh /

RUN pacman -Syu --noconfirm --needed python sudo git namcap wget &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
	mkdir /buildout && chown -R builder:builder /buildout &&\
	chmod +x /buildpkg.py /entrypoint.sh &&\
	cd /tmp &&\
	wget -O paru.tar.gz "https://github.com/Morganamilo/paru/releases/download/v1.11.2/paru-v1.11.2-x86_64.tar.zst" &&\
	tar -xf paru.tar.gz &&\
	cp ./paru /opt/paru &&\
	rm -rf paru.tar.gz paru paru.conf man completions locale &&\
	pacman -Rns --noconfirm wget

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
