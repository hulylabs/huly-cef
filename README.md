## Crates

| Crate Name            | Description                                                                 |
|-----------------------|-----------------------------------------------------------------------------|
| `huly-cef`            | Core library for interacting with the Chromium Embedded Framework (CEF)     |
| `huly-cef-helper`     | macOS-specific helper app required for proper bundle packaging              |
| `huly-cef-tools`      | Utility crate providing tools and helpers for building `huly-cef` apps      |
| `huly-cef-websockets` | WebSocket-based server for streaming and interacting with CEF browser views |


## Launch Instructions For Huly CEF
Follow these steps to build and run Huly CEF:

1. **Download CEF Artifacts**  
   First, download the necessary CEF artifacts from the following link:  
   [CEF Artifacts](https://github.com/hulylabs/cef-ui/releases/tag/cef-artifacts-131).

2. **Set Up Environment Variables**  
   Once you have the CEF artifacts, set the `CEF_ARTIFACTS_DIR` environment variable to the path of the CEF Artifacts. You can do this with the following commands:
   ```bash
   export CEF_ARTIFACTS_DIR=/path/to/cef/libraries
   ```
3. **Build Huly CEF**  
   To build Huly CEF, use the following command:
   ```bash
   cargo run --bin huly-cef-build --release -- --profile release
   ```
4. **Run Huly CEF**  
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