FROM rust:latest

WORKDIR /workspace

RUN apt-get update && apt-get install -y \
    pkg-config \
    libx11-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libssl-dev

COPY . .

CMD ["cargo", "check"]
