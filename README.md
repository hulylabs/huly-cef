## Build Instructions (For Windows and Linux only)
Follow these steps to build and run CEF:

1. **Download CEF Artifacts**  
   First, download the necessary CEF artifacts from the following link:  
   [CEF Artifacts](https://github.com/hytopiagg/cef-ui/releases/tag/cef-artifacts-v0.1.0).

2. **Set Up Environment Variables**  
   Once you have the CEF artifacts, set the `CEF_ARTIFACTS_DIR` environment variable to the path of the CEF Artifacts. You can do this with the following commands:
   ```bash
   export CEF_ARTIFACTS_DIR=/path/to/cef/libraries
   ```
3. **Run CEF**  
   To launch CEF, use the following command:
   ```bash
   cargo run
   ```