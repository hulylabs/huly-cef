# Huly CEF

A Rust-based framework for building desktop applications using the Chromium Embedded Framework (CEF).

## Overview

Huly CEF provides a set of Rust crates and tools for creating cross-platform desktop applications with web technologies. It includes WebSocket-based communication, instance management, and TypeScript client libraries.

## Prerequisites

- Rust (latest stable version)
- Node.js and npm (for TypeScript client)
- Docker (optional, for running the manager in a container)

## Project Structure

| Crate Name            | Description                                                                 |
|-----------------------|-----------------------------------------------------------------------------|
| `huly-cef`            | Core library for interacting with the Chromium Embedded Framework (CEF)     |
| `huly-cef-helper`     | macOS-specific helper app required for proper bundle packaging              |
| `huly-cef-tools`      | Utility crate providing tools and helpers for building `huly-cef` apps      |
| `huly-cef-websockets` | WebSocket-based server for streaming and interacting with CEF browser views |
| `huly-cef-manager`    | A RESTful server that manages Huly CEF instances                           |


## Huly CEF Websockets

## TODO: describe arguments

To build and run Huly CEF Websockets, use:
1. **Build Huly CEF Websockets**  
   ```bash
   cargo run --bin huly-cef-build --release -- --profile release
   ```

2. **Run Huly CEF Websockets**  
   **Linux**:
   ```bash
   ./target/release/huly-cef-websockets
   ```

   **Windows:**
   ```bash
   ./target/release/huly-cef-websockets.exe
   ```

   **macOS:**
   ```bash
   ./target/release/huly-cef-websockets.app/Contents/MacOS/huly-cef-websockets
   ```

## CEF Client (Development Only)

1. **Build The Package**  
   To build the package, use the following command:
   ```bash
   cd ./packages/cef-client
   npm install
   npm run build
   ```
2. **Link The Package**  
   To link the package, use the following command:
   ```bash
   npm link
   ```
3. **Use The Package In Your Project**  
   To use the package in your project, run the following command in the project's folder:
   ```bash
   npm link cef-client
   ```

4. **Publish The Package**
   To publish the package, use the following command:
   ```bash
   npm publish
   ```

## Huly CEF Manager

## TODO: describe arguments

### Running Huly CEF Manager Locally
1. **Build Huly CEF Websockets**  
   ```bash
   cargo run --bin huly-cef-build --release -- --profile release
   ```

2. **Run Huly CEF Manager**  
   **Linux**:
   ```bash
   cargo run --bin huly-cef-manager -- --cef-exe=target/release/huly-cef-websockets
   ```

   **Windows:**
   ```bash
   cargo run --bin huly-cef-manager -- --cef-exe=target/release/huly-cef-websockets.exe
   ```

   **macOS:**
   ```bash
   cargo run --bin huly-cef-manager -- --cef-exe=target/release/huly-cef-websockets.app/Contents/MacOS/huly-cef-websockets
   ```

### Running Huly CEF Manager in a Docker container

```bash
# Build the Docker image
docker build -f Dockerfile.manager . -t huly-cef-manager

# Run the container
docker run \
   -e MANAGER_PORT=3001 \
   -e PORT_RANGE=40000-40100 \
   -e HOST=localhost \
   -p 3001:3001 \
   -p 40000-40100:40000-40100 \
   -v /path/to/cefcache/on/host:/cefcache \
   --rm huly-cef-manager
```

### API Usage Example

```javascript
// Create a CEF instance and get its address
const response = await fetch("http://localhost:3000/profiles/<profile-id>/cef");
const json = await response.json();
const address = json.data.address;

// Connect and control the browser
const browser = await connect(address);
const tab = await browser.openTab({ 
  url: "https://google.com", 
  wait_until_loaded: true 
});

let title = await tab.title();
console.log("Page title: ", title);
```