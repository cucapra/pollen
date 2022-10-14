# USAGE
# With this Dockerfile in working directory,
# docker build -t username/imagename .
# (note the period at the end)
# docker run -it username/imagename /bin/bash

# Start with latest Calyx image
FROM ghcr.io/cucapra/calyx:latest

# return to root directory
WORKDIR /root

# Install ODGI
# dependencies:
RUN apt install -y build-essential cmake python3-distutils python3-dev libjemalloc-dev
# clone:
RUN git clone --recursive https://github.com/pangenome/odgi.git
# build:
WORKDIR /root/odgi
RUN cmake -H. -Bbuild && cmake --build build -- -j7
# return to root directory
WORKDIR /root

# Add ODGI to paths
ENV PATH="/root/odgi/bin:$PATH"
ENV PYTHONPATH=$PYTHONPATH:/root/odgi/lib
ENV LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so.2

# Install Calyx-Pangenome (TODO-AM: rename when repo is renamed)
# dependencies:
RUN pip install --user turnt

# clone:
RUN git clone https://github.com/cucapra/calyx-pangenome.git
# build:
WORKDIR /root/calyx-pangenome
RUN make fetch

# return to root directory
WORKDIR /root
