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
ENV FLIT_ROOT_INSTALL=1

# Install Pollen
# dependencies:
RUN git clone https://github.com/cucapra/turnt.git
WORKDIR /root/turnt
RUN flit install --symlink --user
WORKDIR /root

# good to have:
RUN apt install emacs -y
RUN apt install vim -y

# clone:
RUN git clone https://github.com/cucapra/pollen.git
# build:
WORKDIR /root/pollen
RUN make fetch
RUN make og

# return to root directory
WORKDIR /root

