# Use an official Rust runtime as a parent image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/grammarbot

# Copy the Cargo.toml and src files into the container
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src

# Build your Rust application
RUN cargo build --release

# Install SurrealDB and run
CMD ["curl -sSf https://install.surrealdb.com | sh"]
CMD ["surreal start file://.db"]

# Specify the command to run your application
CMD ["./target/release/your_executable"]
