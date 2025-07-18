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


## Launch Instructions For Huly CEF Websockets
Follow these steps to build and run Huly CEF:

1. **Build Huly CEF Websockets**  
   To build Huly CEF, use the following command:
   ```bash
   cargo run --bin huly-cef-build --release -- --profile release
   ```
2. **Run Huly CEF Websockets**  
   Linux:
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

## Build Instructions For TypeScript CEF Client (For Development)

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

The manager provides a RESTful API for managing multiple CEF instances.

### Running with Docker

```bash
# Build the Docker image
docker build -f Dockerfile.manager . -t huly-cef-manager

# Run the container
docker run -p 3000:3000 -p 40000-40200:40000-40200 -v /path/to/cefcache/on/host:/cefcache --rm huly-cef-manager --port-range 40000-40200
```

### API Usage Example

```javascript
// Create a CEF instance and get its address
const response = await fetch("http://localhost:3000/instances/id");
const address = await response.text();

// Connect and control the browser
const browser = await connect(address);
const tab = await browser.openTab({ 
  url: "https://google.com", 
  wait_until_loaded: true 
});

let title = await tab.title();
console.log("Page title: ", title);
```