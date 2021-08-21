FROM archlinux/archlinux:base

COPY testpkg.py entrypoint.sh /

RUN pacman -Syu --noconfirm &&\
 chmod +x /testpkg.py /entrypoint.sh

ENTRYPOINT [ "/entrypoint.sh" ]
