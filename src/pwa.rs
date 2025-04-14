use std::io::{Read, Write};

use crate::config::Config;
use std::error::Error;

pub fn init_pwa(config: &Config) -> Result<(), Box<dyn Error>> {
    let index = config.root.join(&config.index);
    info!("PWA: {:?}", index.file_name());
    let mut file = std::fs::File::open(&index)?;
    let mut index_html = String::new();
    file.read_to_string(&mut index_html)?;

    // insert mainfest
    let manifest_link =
        "<link crossorigin=\"use-credentials\" rel=\"manifest\" href=\"manifest.json\">\n";
    if !index_html.contains(manifest_link) {
        let manifest_marker = "<meta charset=\"UTF-8\" />";
        let insert_index = index_html.find(manifest_marker).ok_or("marker not find")?;
        index_html.insert_str(insert_index, manifest_link);
    }

    //insert serviceWorker
    let service_insert = "    <script>\n        if (typeof navigator.serviceWorker !== 'undefined') {\n            navigator.serviceWorker.register('sw.js')\n        }\n    </script>\n";
    if !index_html.contains(service_insert) {
        let service_marker = "<body>\n\t<div id=\"init-screen\">";
        let marker_prefix_len = "<body>\n\t".len();
        let insert_index_base = index_html
            .find(service_marker)
            .ok_or("Failed to find marker in index.html")?;
        let insert_index = insert_index_base + marker_prefix_len;
        index_html.insert_str(insert_index, service_insert);
    }

    let style_insert =
        "<style>\n    body {\n        overscroll-behavior-y: none;\n    }\n</style>\n";
    if !index_html.contains(style_insert) {
        let style_marker = "\n</head>";
        let insert_index = index_html.find(style_marker).ok_or("marker not find")?;
        info!("inserted style");
        index_html.insert_str(insert_index, style_insert);
    }

    let mut file = std::fs::File::create(&index)?;
    file.write_all(index_html.as_bytes())?;

    Ok(())
}
