FROM rust:1.84.0-slim-bookworm
RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y git
RUN rustup component add rustfmt clippy
CMD ["sleep", "infinity"]