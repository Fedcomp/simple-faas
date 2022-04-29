FROM rust:1.60 as build

RUN groupadd -g 9999 simple-faas
RUN useradd simple-faas -u 9999 -g 9999 -m -d /opt/simple-faas
USER simple-faas
WORKDIR /opt/simple-faas

COPY --chown=simple-faas:simple-faas Cargo.toml Cargo.lock /opt/simple-faas/
COPY --chown=simple-faas:simple-faas simple-faas /opt/simple-faas/simple-faas/
COPY --chown=simple-faas:simple-faas simple-faas-docker /opt/simple-faas/simple-faas-docker/
RUN cargo build --release

FROM rust:1.60
RUN apt-get update
# Purely for production debugging purposes
RUN apt-get install -y docker.io
RUN groupadd -g 9999 simple-faas
RUN useradd simple-faas -u 9999 -g 9999 -m -d /opt/simple-faas
USER simple-faas
WORKDIR /opt/simple-faas
COPY --from=build --chown=simple-faas:simple-faas \
     /opt/simple-faas/target/release/simple-faas /opt/simple-faas/simple-faas

CMD ["/opt/simple-faas/simple-faas"]
