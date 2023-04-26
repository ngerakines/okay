FROM rust:latest AS builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
ENV USER=okay
ENV UID=10001
RUN adduser --disabled-password --gecos "" --home "/okay" --shell "/sbin/nologin" --no-create-home --uid "${UID}" "${USER}"
WORKDIR /okay
COPY ./ .
RUN cargo build --target x86_64-unknown-linux-musl --release
FROM scratch
LABEL org.opencontainers.image.source="https://github.com/ngerakines/okay"
LABEL org.opencontainers.image.description="A very small HTTP server that has configurable started and ready probes."
LABEL org.opencontainers.image.authors="Nick Gerakines <nick.gerakines@gmail.com>"
LABEL org.opencontainers.image.licenses="MIT"
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
WORKDIR /okay
COPY --from=builder /okay/target/x86_64-unknown-linux-musl/release/okay ./
USER okay:okay
CMD ["/okay/okay"]