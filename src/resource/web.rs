use super::Resource;
use futures::channel::oneshot;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::js_sys::JsString;
use web_sys::HtmlImageElement;

pub struct Image(HtmlImageElement);

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct WasmError(String);

impl From<JsValue> for WasmError {
    fn from(value: JsValue) -> Self {
        Self(String::from(JsString::from(value)))
    }
}

#[cfg(feature = "femtovg")]
impl<'a> TryFrom<&'a Image> for femtovg::ImageSource<'a> {
    type Error = femtovg::ErrorKind;

    fn try_from(value: &'a Image) -> Result<Self, Self::Error> {
        Ok(femtovg::ImageSource::from(&value.0))
    }
}

pub fn load_bytes(path: &str) -> Resource<Vec<u8>> {
    let r = Resource::pending();
    {
        let path = path.to_string();
        let r = r.clone();

        spawn_local(async move {
            let res = async {
                Ok(gloo_net::http::Request::get(&path)
                    .send()
                    .await?
                    .binary()
                    .await?)
            }
            .await
            .map_err(|e: gloo_net::Error| Arc::new(Box::new(e).into()))
            .map(|v| Arc::new(v));

            *r.0.lock().unwrap() = Some(res);
        });
    }
    r
}

async fn load_html_image(src: &str) -> Result<HtmlImageElement, JsValue> {
    let img = HtmlImageElement::new()?;
    let (tx, rx) = oneshot::channel::<Result<(), JsValue>>();

    let tx = Rc::new(RefCell::new(Some(tx)));

    let onload = {
        let tx = tx.clone();
        Closure::once(move || {
            let _ = tx.borrow_mut().take().unwrap().send(Ok(()));
        })
    };
    img.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    let onerror = {
        let tx = tx.clone();
        Closure::once(move |err: JsValue| {
            let _ = tx.borrow_mut().take().unwrap().send(Err(err));
        })
    };
    img.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    img.set_src(src.into());

    rx.await.unwrap()?;
    Ok(img)
}

pub fn load_image(path: &str) -> Resource<Image> {
    let r = Resource::pending();

    {
        let path = path.to_string();
        let r = r.clone();

        spawn_local(async move {
            let res = load_html_image(&path)
                .await
                .map_err(|e| Arc::new(Box::new(WasmError::from(e)).into()))
                .map(|img| Arc::new(Image(img)));

            *r.0.lock().unwrap() = Some(res);
        });
    }

    r
}
