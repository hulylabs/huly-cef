use anyhow::Result;
use cef_ui_util::{link_cef, link_cef_helper};

fn main() -> Result<()> {
    link_cef()?;
    if cfg!(target_os = "macos") {
        link_cef_helper()?;
    }

    Ok(())
}
