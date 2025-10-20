# CEF Manager Development Guide

This guide covers how to develop, test, and deploy the Huly CEF Manager.

## Table of Contents
- [Local Development](#local-development)
- [Building for Production](#building-for-production)
- [Publishing New Versions](#publishing-new-versions)
- [Docker Development](#docker-development)
- [Troubleshooting](#troubleshooting)

## Local Development

You can develop the CEF Manager locally without Docker containers. This provides faster iteration cycles and easier debugging.

### Prerequisites

- Rust (latest stable version)
- Node.js and npm (for TypeScript client)
- macOS/Linux/Windows development environment

### Development Workflow

#### 1. Build CEF Websockets (Required First)

Before running the CEF Manager, you must first build the CEF Websockets component:

```bash
cargo run --bin huly-cef-build
```

This command builds the necessary CEF binaries and websockets server that the manager will control.

#### 2. Run CEF Manager Locally

Once the websockets component is built, you can run the CEF Manager with the appropriate executable path for your platform:

**macOS:**
```bash
cargo run --bin huly-cef-manager -- --cef-exe=target/release/huly-cef-websockets.app/Contents/MacOS/huly-cef-websockets
```

**Linux:**
```bash
cargo run --bin huly-cef-manager -- --cef-exe=target/release/huly-cef-websockets
```

**Windows:**
```bash
cargo run --bin huly-cef-manager -- --cef-exe=target/release/huly-cef-websockets.exe
```

### Configuration Options

The CEF Manager accepts the following command-line arguments (can also be set via environment variables):

- `--cache-dir` / `CACHE_DIR`: Root directory for CEF cache storage (default: `cache`)
- `--cef-exe` / `CEF_EXE`: Path to the CEF executable (required, no default)
- `--port-range` / `PORT_RANGE`: Port range for CEF instances in format START-END (default: `10000-10100`)
- `--host` / `HOST`: Host for CEF servers and Manager (default: `localhost`)
- `--manager-port` / `MANAGER_PORT`: Port for the CEF Manager (default: `3000`)
- `--use-server-size` / `USE_SERVER_SIZE`: Whether to use server size for CEF instances (default: `false`)

Example with custom configuration:
```bash
cargo run --bin huly-cef-manager -- \
  --cef-exe=target/release/huly-cef-websockets \
  --cache-dir=./dev-cache \
  --manager-port=3001 \
  --port-range=40000-40100 \
  --host=0.0.0.0
```

## Building for Production

### Publishing via Docker Branch

To publish a new production version, first update the version number, then create and push a `docker` branch. This will automatically build and publish the Docker image to the registry.

```bash
# 1. Update version in Cargo.toml
# Edit crates/huly-cef-manager/Cargo.toml and update the version field

# 2. Create a new docker branch from your current branch
git checkout -b docker

# 3. Push the branch to trigger the automated build and publish
git push origin docker
```

The published image can then be run with:

```bash
docker run \
  -e CACHE_DIR=/cefcache \
  -e CEF_EXE=/apps/huly-cef-websockets \
  -e PORT_RANGE=40000-40100 \
  -e HOST=localhost \
  -e MANAGER_PORT=3001 \
  -e USE_SERVER_SIZE=false \
  -p 3001:3001 \
  -p 40000-40100:40000-40100 \
  -v /tmp/cefcache:/cefcache \
  --rm <registry>/huly-cef:latest
```

## API Testing

Test the manager API locally:

```bash
# List all profiles
curl http://localhost:3001/profiles

# Create a CEF instance (replace with actual profile ID)
curl -X POST http://localhost:3001/profiles/test-profile/cef

# Destroy a CEF instance
curl -X DELETE http://localhost:3001/profiles/test-profile/cef
```

### CEF Client Usage Example

Once you have a CEF instance running, you can use the cef-client package to interact with it:

```javascript
import { connect } from 'cef-client';

// Create a CEF instance and get its address
const response = await fetch("http://localhost:3001/profiles/my-profile/cef");
const json = await response.json();
const address = json.data.address;

// Connect and control the browser
const browser = await connect(address);

// Open a new tab
const tab = await browser.openTab({ 
  url: "https://google.com", 
  wait_until_loaded: true 
});

// Get page title
let title = await tab.title();
console.log("Page title: ", title);

// Navigate to another page
await tab.navigate("https://github.com");

// Close the tab
await tab.close();
```
