FROM ubuntu:24.04

RUN apt-get update
RUN apt-get install -y curl build-essential libgmp3-dev libmpfr-dev libmpfr-doc libmpc-dev

# 00-environment
RUN apt-get install -y qemu-system nasm

# 11-kernel-crosscompiler

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

# 14-checkpoint

## gdb
RUN \
  curl -O http://ftp.rediris.es/mirror/GNU/gdb/gdb-16.3.tar.gz \
  && tar xzf gdb-16.3.tar.gz \
  && mkdir gdb-build \
  && cd gdb-build \
  && ../gdb-16.3/configure --target=i386-elf --prefix=/usr/local/i386elfgcc --program-prefix=i386-elf- \
  && make \
  && make install \
  && cd .. \
  && rm -rf gdb-16.3 gdb-16.3.tar.gz gdb-build

WORKDIR /app
ENV PATH=${PATH}:/usr/local/i386elfgcc/bin/

