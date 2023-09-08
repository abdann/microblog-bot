####################################################################################################
## Builder
####################################################################################################
FROM --platform=${BUILDPLATFORM} rust:latest AS builder
ARG TARGETPLATFORM
RUN apt update && apt install -y musl-tools musl-dev llvm clang
RUN update-ca-certificates

# Create appuser
ENV USER=microblogbot
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /microblogbot

COPY . .
# Magic environment variables to get "ring" to cross compile
# See https://github.com/briansmith/ring/issues/1414
ENV CC_aarch64_unknown_linux_musl=clang
ENV AR_aarch64_unknown_linux_musl=llvm-ar
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"

RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then ARCHITECTURE=x86_64; elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then ARCHITECTURE=aarch64; else ARCHITECTURE=x86_64; fi \
    && rustup target add ${ARCHITECTURE}-unknown-linux-musl && cargo build --release --target ${ARCHITECTURE}-unknown-linux-musl && mv target/${ARCHITECTURE}-unknown-linux-musl/release/microblog-bot .
####################################################################################################
## Final
####################################################################################################
FROM alpine

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /microblogbot

# Copy our build
COPY --from=builder /microblogbot/microblog-bot .
COPY log4rs-config.yml .

# Use an unprivileged user.
USER microblogbot:microblogbot

ENV IN_CONTAINER='true'
CMD ["/microblogbot/microblog-bot"]
