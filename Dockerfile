
FROM rust:slim as builder
WORKDIR /usr/src/cafe_website
COPY . .
RUN SQLX_OFFLINE=true cargo build -r


FROM debian:bookworm-slim as cafe_website
WORKDIR /var/cafe_website

ENV PORT=443
EXPOSE $PORT

COPY --from=builder /usr/src/cafe_website/target/release/cafe_website /usr/local/bin/cafe_website

ENV CERTS_DIR="/var/cafe_website/certs"
COPY ./certs $CERTS_DIR

COPY --from=builder /usr/src/cafe_website/public ./public

ENTRYPOINT ["cafe_website"]
