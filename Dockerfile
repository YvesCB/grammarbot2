FROM rust:1.71.0 as build
# create a new empty shell project
RUN USER=root cargo new --bin grammarbot2
WORKDIR /grammarbot2

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./logging_config.yaml ./logging_cofing.yaml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/grammarbot2*
RUN cargo build --release

# our final base
FROM rust:1.71.0

# copy the build artifact from the build stage
COPY --from=build /grammarbot2/target/release/grammarbot2 .
COPY ./logging_config.yaml ./logging_config.yaml

# set the startup command to run your binary
CMD ["./grammarbot2"]
