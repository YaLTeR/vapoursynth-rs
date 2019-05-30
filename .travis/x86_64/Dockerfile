FROM ubuntu:xenial

RUN set -ex; \
    apt-get update -yqq; \
    apt-get install -yqq build-essential \
                         curl \
                         python3 \
                         software-properties-common; \
    add-apt-repository -y ppa:djcj/hybrid; \
    apt-get update -yqq; \
    apt-get install -yqq vapoursynth; \
    apt-get clean; \
    ln -s libvapoursynth-script.so.0 /usr/lib/x86_64-linux-gnu/libvapoursynth-script.so
