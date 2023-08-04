FROM rust:alpine

WORKDIR /newsletter

RUN apk update && apk add mold clang

COPY . .

RUN cargo build --release

ENTRYPOINT [ "./target/release/newsletter" ]