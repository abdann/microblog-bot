####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
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

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM alpine

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /microblogbot

# Copy our build
COPY --from=builder /microblogbot/target/x86_64-unknown-linux-musl/release/microblog-bot ./
COPY /log4rs-config.yml .

# Use an unprivileged user.
USER microblogbot:microblogbot

ENV IN_CONTAINER='true'
CMD ["/microblogbot/microblog-bot"]
