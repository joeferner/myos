FROM ubuntu:24.04

RUN apt-get update
RUN apt-get install -y curl build-essential libgmp3-dev libmpfr-dev libmpfr-doc libmpc-dev

# 00-environment
RUN apt-get install -y qemu-system nasm

# 11-kernel-crosscompiler
# ENV PREFIX="/usr/local/i386elfgcc"
# ENV TARGET=i386-elf
# ENV PATH="$PREFIX/bin:$PATH"

## binutils
RUN \
  curl -O https://ftp.gnu.org/gnu/binutils/binutils-2.44.tar.gz \
  && tar xzf binutils-2.44.tar.gz \
  && mkdir binutils-build \
  && cd binutils-build \
  && ../binutils-2.44/configure --target=i386-elf --enable-interwork --enable-multilib --disable-nls --disable-werror --prefix=/usr/local/i386elfgcc \
  && make all \
  && make install \
  && cd .. \
  && rm -rf binutils-2.44 binutils-2.44.tar.gz binutils-build

## gcc
RUN \
  curl -O https://ftp.gnu.org/gnu/gcc/gcc-13.4.0/gcc-13.4.0.tar.gz \
  && tar xzf gcc-13.4.0.tar.gz \
  && mkdir gcc-build \
  && cd gcc-build \
  && ../gcc-13.4.0/configure --target=i386-elf --prefix=/usr/local/i386elfgcc --disable-nls --disable-libssp --enable-languages=c --without-headers \
  && make all-gcc \
  && make all-target-libgcc \
  && make install-gcc \ 
  && make install-target-libgcc \
  && cd .. \
  && rm -rf gcc-13.4.0 gcc-13.4.0.tar.gz gcc-build

  WORKDIR /app
  ENV PATH=${PATH}:/usr/local/i386elfgcc/bin/
  