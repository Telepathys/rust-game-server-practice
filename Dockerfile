FROM rust:latest
COPY . /app
WORKDIR /app
RUN cargo build --release

EXPOSE 1111
CMD ["cargo","run","--bin","rust-game-server-practice"]