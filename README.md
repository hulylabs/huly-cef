## Crates

| Crate Name            | Description                                                                 |
|-----------------------|-----------------------------------------------------------------------------|
| `huly-cef`            | Core library for interacting with the Chromium Embedded Framework (CEF)     |
| `huly-cef-helper`     | macOS-specific helper app required for proper bundle packaging              |
| `huly-cef-tools`      | Utility crate providing tools and helpers for building `huly-cef` apps      |
| `huly-cef-websockets` | WebSocket-based server for streaming and interacting with CEF browser views |
| `huly-cef-manager`    | A RESTful server that manages Huly CEF instances                            |


## Launch Instructions For Huly CEF
Follow these steps to build and run Huly CEF:

1. **Build Huly CEF**  
   To build Huly CEF, use the following command:
   ```bash
   cargo run --bin huly-cef-build --release -- --profile release
   ```
2. **Run Huly CEF**  
   Linux:
   ```bash
   ./target/release/huly-cef-websockets
   ```

   Windows:
   ```bash
   ./target/release/huly-cef-websockets.exe
   ```

   MacOS:
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

## Interacting with Huly CEF Manager

The `huly-cef-manager` provides a RESTful API for managing CEF instances. Here's how to interact with it:

1. **Start the Manager**  
   First, ensure the huly-cef-manager is running on the default port (3000).

2. **Create CEF Instance and get its Address**  
   Fetch the address of a CEF instance by its ID:
   ```javascript
   const cef = await fetch("http://localhost:3000/instances/id");
   const address = await cef.text();
   ```

3. **Connect and Control Browser**  
   Use the address to connect and control the browser:
   ```javascript
   const browser = await connect(address);
   let tab = await browser.openTab({ url: "https://google.com", wait_until_loaded: true });

   let title = await tab.title();
   console.log("Page title: ", title);
   ```