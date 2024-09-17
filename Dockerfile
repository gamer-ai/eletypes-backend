# Use a base image with the latest version of Rust installed for the build stage
FROM rust:latest as build

# Create a new empty shell project
RUN USER=root cargo new --bin eletypes-backend

# Set the working directory in the container
WORKDIR /eletypes-backend

# Copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Cache dependencies to optimize build times
RUN cargo build --release
RUN rm src/*.rs

# Copy the local application code into the container
COPY . .

# Build the application for release
RUN cargo build --release

# Use a minimal base image for the final stage
FROM gcr.io/distroless/cc

# Copy the build artifact from the build stage
COPY --from=build /eletypes-backend/target/release/eletypes-backend /usr/local/bin/eletypes-backend

# Specify the command to run when the container starts
CMD ["/usr/local/bin/eletypes-backend"]
