ARG PLATFORM=amd64
ARG CROSS_BUILD_TARGET=x86_64-unknown-linux-gnu
FROM ${PLATFORM}/alpine:latest
ARG CROSS_BUILD_TARGET
RUN echo "Creating container based on ${PLATFORM}/alpine:latest"
RUN echo "Using Rust binaries from ${CROSS_BUILD_TARGET}"

COPY ./target/${CROSS_BUILD_TARGET}/release/datetime /datetime

ENV RUST_LOG=info
CMD ["./datetime"]
