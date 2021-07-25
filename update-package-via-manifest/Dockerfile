FROM node:latest AS builder

WORKDIR /src/update-package-via-manifest

COPY . .

RUN npm i && npm run compile

FROM archlinux/archlinux:base-devel

COPY --from=builder /src/update-package-via-manifest/lib/index.js /index.js

COPY entrypoint.sh /

RUN pacman -Syu --noconfirm &&\
 pacman -S --noconfirm --needed sudo nodejs git pacman-contrib &&\
 useradd builder -m &&\
 passwd -d builder &&\
 printf 'builder ALL=(ALL) ALL\n' | tee -a /etc/sudoers &&\
 chmod +x /entrypoint.sh

USER builder

ENTRYPOINT [ "/entrypoint.sh" ]
