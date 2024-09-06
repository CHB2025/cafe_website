
FROM rust:bullseye as builder
# install tailwind cli
RUN wget https://github.com/tailwindlabs/tailwindcss/releases/download/v3.4.3/tailwindcss-linux-x64; \
    chmod +x tailwindcss-linux-x64; \
    mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss;

WORKDIR /usr/src/cafe_website
COPY . .
RUN SQLX_OFFLINE=true cargo build -r


FROM debian:bookworm-slim as cafe_website
WORKDIR /var/cafe_website

ENV PORT=443
EXPOSE $PORT

COPY --from=builder /usr/src/cafe_website/target/release/cafe_website /usr/local/bin/cafe_website
COPY --from=builder /usr/src/cafe_website/public ./public

ENTRYPOINT ["cafe_website"]
