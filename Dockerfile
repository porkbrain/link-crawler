FROM rust:1.31

WORKDIR /home/ec2-user/run
COPY . .

RUN rustup update nightly
RUN rustup default nightly
RUN cargo install --path .

EXPOSE 8000

CMD ["cargo", "run", "--release"]
