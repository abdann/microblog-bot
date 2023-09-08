####################################################################################################
## Builder
####################################################################################################
FROM --platform=amd64 rust:latest AS build-amd64

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

COPY ./ .

FROM --platform=arm64 rust:latest AS build-arm64

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

COPY ./ .

FROM build-${TARGETARCH} AS builder
RUN cargo build --release
# FROM rust:latest AS builder

# RUN apt update && apt install -y musl-tools musl-dev
# RUN update-ca-certificates

# # Create appuser
# ENV USER=microblogbot
# ENV UID=10001

# RUN adduser \
#     --disabled-password \
#     --gecos "" \
#     --home "/nonexistent" \
#     --shell "/sbin/nologin" \
#     --no-create-home \
#     --uid "${UID}" \
#     "${USER}"


# WORKDIR /microblogbot

# COPY ./ .

# FROM --platform=amd64 builder AS build-amd64

# RUN rustup target add x86_64-unknown-linux-musl
# RUN cargo build --target x86_64-unknown-linux-musl --release

# FROM --platform=arm64 builder AS build-arm64
# RUN rustup target add aarch64-unknown-linux-musl
# RUN cargo build --target aarch64-unknown-linux-musl --release

# ####################################################################################################
# ## Final image
# ####################################################################################################
FROM alpine

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /microblogbot

# Copy our build
COPY --from=builder /microblogbot/target/release/microblog-bot .
COPY log4rs-config.yml .

# Use an unprivileged user.
USER microblogbot:microblogbot

ENV IN_CONTAINER='true'
CMD ["/microblogbot/microblog-bot"]
