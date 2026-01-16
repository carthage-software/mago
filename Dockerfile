FROM rust:1.91.1-bookworm

# Add SURY repository for PHP 8.5
RUN apt-get update && apt-get install -y \
    curl \
    lsb-release \
    ca-certificates \
    gnupg \
    && curl -fsSL https://packages.sury.org/php/apt.gpg | gpg --dearmor -o /usr/share/keyrings/sury-php-archive-keyring.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/sury-php-archive-keyring.gpg] https://packages.sury.org/php/ $(lsb_release -sc) main" | tee /etc/apt/sources.list.d/sury-php.list \
    && rm -rf /var/lib/apt/lists/*

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    php8.5-cli \
    composer \
    && rm -rf /var/lib/apt/lists/*

# Use system OpenSSL instead of building from source
ENV OPENSSL_NO_VENDOR=1

# Install just task runner
RUN cargo install just

# Install cargo-watch for development watch mode
RUN cargo install cargo-watch

# Install wasm-pack for WebAssembly builds
RUN cargo install wasm-pack

# Install nightly Rust for advanced checks
RUN rustup toolchain install nightly

# Set working directory
WORKDIR /workspace

# Default command
CMD ["/bin/bash"]
