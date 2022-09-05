FROM rust:1.63.0 as build-env
WORKDIR /app
COPY . /app
RUN apt-get update && apt-get -y install wget build-essential cmake autoconf automake libtool m4 && cargo build --release && wget https://bootstrap.pypa.io/get-pip.py

FROM mwader/static-ffmpeg:latest as ffmpeg


FROM python:3
COPY --from=build-env /app/target/release/thoria /
COPY --from=build-env /app/get-pip.py /
COPY --from=ffmpeg /ffmpeg /usr/bin
RUN python3 get-pip.py
RUN python3 -m pip install --no-cache-dir --force-reinstall yt-dlp
ENTRYPOINT ["/thoria"]