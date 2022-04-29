FROM rust:1.60

RUN apt-get update
RUN apt-get install -y docker.io

ARG UID
ARG GID
RUN groupadd -g $GID vscode || true
RUN useradd vscode -u $UID -g $GID -m
USER vscode
WORKDIR /home/vscode/workspace

# Must be installed under user
RUN rustup component add rustfmt
RUN cargo install cargo-edit

CMD ["sleep", "infinity"]
