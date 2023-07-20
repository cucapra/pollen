# USAGE
# With this Dockerfile in working directory,
# docker build -t username/imagename .
# (note the period at the end)
# docker run -it --rm username/imagename

# Start with latest Calyx image
FROM ghcr.io/cucapra/calyx:latest

# Go to the root directory
WORKDIR /root

# Install ODGI
# Dependencies:
RUN apt install -y build-essential cmake python3-distutils python3-dev libjemalloc-dev
# Clone:
RUN git clone --recursive https://github.com/pangenome/odgi.git
# Build:
WORKDIR /root/odgi
RUN cmake -H. -Bbuild && cmake --build build -- -j7
# Return to root directory
WORKDIR /root

# Add ODGI to paths
ENV PATH="/root/odgi/bin:$PATH"
ENV PYTHONPATH=$PYTHONPATH:/root/odgi/lib
ENV LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so.2
ENV FLIT_ROOT_INSTALL=1

# Install Pollen's dependencies:
RUN git clone https://github.com/cucapra/turnt.git
WORKDIR /root/turnt
RUN flit install -s --user
WORKDIR /root

# Good to have:
RUN apt install emacs -y
RUN apt install vim -y

# Clone and build Pollen:
RUN git clone https://github.com/cucapra/pollen.git
WORKDIR /root/pollen
RUN make fetch
RUN make og
WORKDIR /root/pollen/pollen_py
RUN flit install -s --user
WORKDIR /root/pollen/mygfa
RUN flit install -s --user
WORKDIR /root/pollen/slow_odgi
RUN flit install -s --user

# return to the Pollen directory
WORKDIR /root/pollen