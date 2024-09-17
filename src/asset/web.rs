use super::{Loadable, Raw};
use futures::channel::oneshot;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
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

impl Loadable for Raw {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Raw(gloo_net::http::Request::get(&path)
            .send()
            .await?
            .binary()
            .await?))
    }
}

impl Loadable for Image {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        let img = HtmlImageElement::new().map_err(|e| WasmError::from(e))?;
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

        img.set_src(path.into());

        rx.await.unwrap().map_err(|e| WasmError::from(e))?;
        Ok(Image(img))
    }
}
