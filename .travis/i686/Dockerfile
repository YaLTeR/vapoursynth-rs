FROM ubuntu:xenial

RUN set -ex; \
    dpkg --add-architecture i386; \
    apt-get update -yqq; \
    apt-get install -yqq build-essential \
                         curl \
                         gcc-multilib \
                         python3 \
                         software-properties-common; \
    add-apt-repository -y ppa:djcj/hybrid; \
    apt-get update -yqq; \
    apt-get install -yqq vapoursynth:i386; \
    apt-get clean; \
    ln -s libvapoursynth-script.so.0 /usr/lib/i386-linux-gnu/libvapoursynth-script.so
