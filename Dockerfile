FROM rust:1.63.0-alpine3.16 as build-env
WORKDIR /app
COPY . /app
RUN apk update && apk add musl-dev wget make cmake autoconf automake libtool m4 && wget https://bootstrap.pypa.io/get-pip.py
RUN cargo build --release

FROM gcr.io/distroless/python3-debian11
COPY --from=build-env /app/target/release/thoria /
COPY --from=build-env /app/get-pip.py /
RUN python3 get-pip.py
RUN python3 -m pip install --no-cache-dir --force-reinstall yt-dlp
ENTRYPOINT ["/thoria"]