#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[allow(warnings)]
mod bindings;
pub mod owned;

use std::ffi::CString;
use std::io::Write;

use crate::bindings::{free_and_destroy_model, load_model, save_model};
use eyre::{eyre, Report, WrapErr};

#[derive(Debug)]
pub struct Model {
    model: *mut bindings::model,
    bias: f64,
    num_features: i32,

    owned: bool,
}

unsafe impl Send for Model {}
unsafe impl Sync for Model {}

impl Model {
    pub fn load_from_file(path: &str) -> Result<Self, Report> {
        let c_path = CString::new(path).wrap_err("invalid c-string")?;

        let model = unsafe { load_model(c_path.as_ptr()) };

        unsafe { Self::new(model) }
    }

    pub fn load_from_str(def: &str) -> Result<Self, Report> {
        let mut temp = tempfile::NamedTempFile::new().wrap_err("error creating tempfile")?;
        temp.write_all(def.as_bytes())
            .wrap_err("error writing to tempfile")?;

        Model::load_from_file(temp.path().to_str().unwrap())
    }

    unsafe fn new(model: *mut bindings::model) -> Result<Self, Report> {
        if model.is_null() {
            return Err(eyre!("null model pointer"));
        }

        Ok(Self {
            model,
            bias: (*model).bias,
            num_features: (*model).nr_feature,
            owned: false,
        })
    }

    pub(crate) unsafe fn new_owned(model: *mut bindings::model) -> Self {
        Self {
            model,
            bias: (*model).bias,
            num_features: (*model).nr_feature,
            owned: true,
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Report> {
        let c_path = CString::new(path).wrap_err("invalid c-string")?;

        let res = unsafe { save_model(c_path.as_ptr(), self.model) };

        if res == 0 {
            Ok(())
        } else {
            Err(eyre!("error saving model, code: {res}"))
        }
    }

    pub fn bias(&self) -> f64 {
        self.bias
    }

    pub fn num_features(&self) -> i32 {
        self.num_features
    }

    pub fn predict(&self, features: &[f64]) -> f64 {
        let mut fns: Vec<bindings::feature_node> = features
            .iter()
            .enumerate()
            .map(|(idx, val)| bindings::feature_node {
                index: (idx + 1) as _,
                value: *val,
            })
            .collect();

        if self.bias >= 0.0 {
            fns.push(bindings::feature_node {
                index: (features.len() + 1) as _,
                value: self.bias,
            });
        }

        fns.push(bindings::feature_node {
            index: -1,
            value: 0f64,
        });

        unsafe { bindings::predict(self.model, fns.as_ptr()) }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        if !self.owned {
            unsafe { free_and_destroy_model(&mut self.model) }
        }
    }
}
