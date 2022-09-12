FROM rust:1.63.0 as build-env
WORKDIR /app
COPY . /app
RUN apt-get update && apt-get -y install wget build-essential cmake autoconf automake libtool m4 && wget https://bootstrap.pypa.io/get-pip.py
RUN cargo build --release

FROM mwader/static-ffmpeg:latest as ffmpeg

FROM gcr.io/distroless/python3-debian11
COPY --from=build-env /app/target/release/thoria /
COPY --from=build-env /app/get-pip.py /
COPY --from=ffmpeg /ffmpeg /usr/bin
RUN python3 get-pip.py
RUN python3 -m pip install --no-cache-dir --force-reinstall yt-dlp
ENTRYPOINT ["/thoria"]