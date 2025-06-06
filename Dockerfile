FROM rust

RUN apt-get update && apt-get install -y \
    libnss3 \
    libnss3-tools \
    libnspr4 \
    libdbus-1-3 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libxkbcommon0 \
    libasound2 \
    libatspi2.0-0 

COPY . /huly-cef

WORKDIR /huly-cef

RUN cargo build --bin huly-cef-websockets --release

CMD ["./target/release/huly-cef-websockets", "--headless"]
