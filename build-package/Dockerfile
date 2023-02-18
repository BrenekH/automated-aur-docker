FROM archlinux/archlinux:base-devel

COPY buildpkg.py entrypoint.sh /

RUN pacman -Syu --noconfirm --needed python sudo git namcap wget &&\
	useradd builder -m &&\
	passwd -d builder &&\
	printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
	mkdir /buildout && chown -R builder:builder /buildout &&\
	chmod +x /buildpkg.py /entrypoint.sh &&\
	cd /tmp &&\
	wget -O yay.tar.gz "https://github.com/Jguer/yay/releases/download/v11.3.2/yay_11.3.2_x86_64.tar.gz" &&\
	tar -xzf yay.tar.gz &&\
	cp yay_*/yay /opt/yay &&\
	rm -rf yay* &&\
	pacman -Rns wget

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
