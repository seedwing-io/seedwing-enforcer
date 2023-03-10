// From: https://github.com/silvanshade/tower-lsp-web-demo/blob/main/crates/browser/src/lib.rs

use futures::stream::TryStreamExt;
use seedwing_enforcer_lsp_common::backend::Backend;
use tower_lsp::{LspService, Server};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::stream::JsStream;

#[wasm_bindgen]
pub struct ServerConfig {
    /*
    into_server: js_sys::AsyncIterator,
    from_server: web_sys::WritableStream,
     */
}

#[wasm_bindgen]
impl ServerConfig {
    /*
    #[wasm_bindgen(constructor)]
    pub fn new(into_server: js_sys::AsyncIterator, from_server: web_sys::WritableStream) -> Self {
        Self {
            into_server,
            from_server,
        }
    }*/

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}

// NOTE: we don't use web_sys::ReadableStream for input here because on the
// browser side we need to use a ReadableByteStreamController to construct it
// and so far only Chromium-based browsers support that functionality.

// NOTE: input needs to be an AsyncIterator<Uint8Array, never, void> specifically
#[wasm_bindgen]
pub async fn serve(config: ServerConfig) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"server::serve".into());

    /*
        let ServerConfig {
            into_server,
            from_server,
        } = config;

        let input = JsStream::from(into_server);
        let input = input
            .map_ok(|value| {
                value
                    .dyn_into::<js_sys::Uint8Array>()
                    .expect("could not cast stream item to Uint8Array")
                    .to_vec()
            })
            .map_err(|_err| std::io::Error::from(std::io::ErrorKind::Other))
            .into_async_read();

        let output = JsCast::unchecked_into::<wasm_streams::writable::sys::WritableStream>(from_server);
        let output = wasm_streams::WritableStream::from_raw(output);
        let output = output.try_into_async_write().map_err(|err| err.0)?;

        let (service, socket) = LspService::build(Backend::new).finish();
        Server::new(input, output, socket).serve(service).await;
    */

    Ok(())
}
