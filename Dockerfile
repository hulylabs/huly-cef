FROM rust AS build

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

FROM debian:bookworm-slim AS runtime
COPY --from=build /huly-cef/target/release/huly-cef-websockets /app/huly-cef-websockets
COPY --from=build /huly-cef/cef /app/cef

RUN apt-get update && apt-get install -y \
    libnss3 \
    libx11-xcb1 \
    libxcomposite1 \
    libxcursor1 \
    libxdamage1 \
    libxext6 \
    libxfixes3 \
    libxi6 \
    libxrandr2 \
    libxrender1 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdbus-1-3 \
    libgdk-pixbuf2.0-0 \
    libglib2.0-0 \
    libgtk-3-0 \
    libnspr4 \
    libpango-1.0-0 \
    libpangocairo-1.0-0 \
    libasound2 \
    libxss1 \
    libxtst6 \
    libnss3 \
    libegl1 \
    libgbm1 \
    libdrm2 \
    fonts-liberation \
    libudev1 \
    xvfb

ADD https://github.com/krallin/tini/releases/download/v0.19.0/tini /tini

RUN chmod +x /tini
ENTRYPOINT ["/tini", "--", "xvfb-run", "-a", "/app/huly-cef-websockets", "--no-sandbox", "--disable-gpu"]
