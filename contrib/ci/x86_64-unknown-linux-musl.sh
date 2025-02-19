#!/bin/bash

set -x

dpkg --add-architecture $CROSS_DEB_ARCH
apt-get update
apt-get --assume-yes install \
  perl \
  clang llvm pkg-config nettle-dev gcc-multilib libc6-dev \
  libgmp-dev libgmp3-dev
