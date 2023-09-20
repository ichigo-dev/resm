FROM ubuntu:20.04

# Environment variables
ARG USER
ARG HOME
ENV USER=${USER}
ENV HOME=${HOME}

# Install dependencies
RUN apt-get update && \
	apt-get install -y ssh sudo
RUN apt-get install -y git

# Create user
RUN useradd -ms /bin/bash ${USER}
RUN echo "${USER} ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers
USER ${USER}

# Copy files
COPY ./resm /usr/local/bin/

# Setup
RUN git config --global core.autocrlf false && \
	sudo chmod +x /usr/local/bin/resm
WORKDIR ${HOME}
