# Use a base image with the latest version of Rust installed
FROM rust:latest as build

# create a new empty shell project
RUN USER=root cargo new --bin eletypes-backend

# Set the working directory in the container
WORKDIR /eletypes-backend

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy the local application code into the container
COPY . .

# build for release
# RUN rm ./target/release/deps/eletypes-backend*
RUN cargo build --release

# our final base
FROM rust:latest

# copy the build artifact from the build stage
COPY --from=build /eletypes-backend/target/release/eletypes-backend .

# Specify the command to run when the container starts
CMD ["./eletypes-backend"]
