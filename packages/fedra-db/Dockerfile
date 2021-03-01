ARG PLATFORM=amd64
ARG CROSS_BUILD_TARGET=x86_64-unknown-linux-gnu
FROM ${PLATFORM}/debian:buster-slim
ARG CROSS_BUILD_TARGET
RUN echo "Creating container based on ${PLATFORM}/debian:buster-slim"
RUN echo "Using Rust binaries from ${CROSS_BUILD_TARGET}"

RUN apt-get update
COPY ./target/${CROSS_BUILD_TARGET}/release/fedra-db /fedra-db

ENV RUST_LOG=info,api
CMD ["./fedra-db"]
