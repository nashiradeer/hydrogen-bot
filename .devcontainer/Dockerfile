FROM rust:1.85.0-slim-bookworm
RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y git
#RUN apk -U upgrade
#RUN apk add --no-cache git lldb musl-dev
RUN rustup component add rustfmt clippy
CMD ["sleep", "infinity"]