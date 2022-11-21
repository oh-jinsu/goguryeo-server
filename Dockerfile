FROM rust:1.64-slim AS builder

WORKDIR /dist

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc AS runtime

COPY --from=builder /dist/target/release/mmorpg /

EXPOSE 3000

CMD ["./mmorpg"]