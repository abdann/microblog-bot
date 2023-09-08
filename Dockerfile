####################################################################################################
## Builder
####################################################################################################
FROM --platform=${BUILDPLATFORM} rust:latest AS build
ARG TARGETPLATFORM
RUN apt update && apt install -y musl-tools musl-dev
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
RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then ARCHITECTURE=x86_64; elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then ARCHITECTURE=aarch64; else ARCHITECTURE=x86_64; fi \
    && cargo build --release --target ${ARCHITECTURE}-unknown-linux-musl && mv target/${ARCHITECTURE}-unknown-linux-musl/release/microblog-bot .
####################################################################################################
## Final
####################################################################################################
FROM alpine

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /microblogbot

# Copy our build
COPY --from=builder microblog-bot .
COPY log4rs-config.yml .

# Use an unprivileged user.
USER microblogbot:microblogbot

ENV IN_CONTAINER='true'
CMD ["/microblogbot/microblog-bot"]
