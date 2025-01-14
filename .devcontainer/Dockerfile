FROM rust:1.84.0-alpine3.21
#RUN apt-get update && apt-get upgrade -y
#RUN apt-get install -y git
RUN apk -U upgrade
RUN apk add --no-cache git lldb musl-dev
RUN rustup component add rustfmt clippy
CMD ["sleep", "infinity"]