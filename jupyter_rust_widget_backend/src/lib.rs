use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};

pub trait RustWidget: Send + Sync + 'static {
    type BackendMessage: for<'de> Deserialize<'de> + Send + Sync + 'static;
    type FrontendMessage: Serialize + Send + Sync + 'static;

    fn handle_message(&mut self, event: Self::BackendMessage) -> Option<Self::FrontendMessage>;

    fn esm(&self) -> String;

    fn into_widget<'py, 'module>(
        self,
        py: Python<'py>,
        module: &'module str,
    ) -> PyResult<Bound<'py, PyAny>>
    where
        Self: Sized,
    {
        let some_widget_instance = SomeWidget::new(self);
        let base_bound = Bound::new(py, some_widget_instance)?;
        let current_module = py.import(module)?;
        let rust_widget_class = current_module.getattr("RustWidget")?;
        let widget_instance = rust_widget_class.call1((base_bound,))?;
        Ok(widget_instance)
    }
}

#[pyclass(subclass)]
struct SomeWidget {
    #[pyo3(get, set)]
    pub _esm: String,
    event_handler: Box<dyn FnMut(String) -> Option<String> + Send + Sync + 'static>,
}

impl SomeWidget {
    fn new<W>(mut widget: W) -> SomeWidget
    where
        W: RustWidget,
    {
        SomeWidget {
            _esm: widget.esm(),
            event_handler: Box::new(move |raw_event| {
                let message = serde_json::de::from_str::<W::BackendMessage>(&raw_event).unwrap();
                widget
                    .handle_message(message)
                    .map(|msg| serde_json::to_string(&msg).unwrap())
            }),
        }
    }
}

#[pymethods]
impl SomeWidget {
    fn _wasm_handle_custom_msg(
        &mut self,
        py: Python<'_>,
        widget: Bound<'_, PyAny>,
        // Assume we always get an object from wasm
        // https://anywidget.dev/en/jupyter-widgets-the-good-parts#data-types
        data_from_wasm: Bound<'_, PyDict>,
        _buffers: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let json_module = py.import("json")?;
        let raw_json: String = json_module
            .call_method1("dumps", (data_from_wasm,))?
            .extract()?;

        if let Some(msg) = (self.event_handler)(raw_json) {
            widget.call_method1("send", (msg,))?;
        }

        Ok(())
    }
}

pub fn inject_rust_widget(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let ctx = PyDict::new(py);
    ctx.set_item("anywidget", py.import("anywidget")?)?;

    let py_code = cr#"
class RustWidget(anywidget.AnyWidget):
    def __init__(self, rust_handler, *args, **kwargs):
        self._esm = rust_handler._esm
        anywidget.AnyWidget.__init__(self, *args, **kwargs)
        self.on_msg(rust_handler._wasm_handle_custom_msg)
"#;

    py.run(py_code, Some(&ctx), None)?;
    let widget_class = ctx.get_item("RustWidget")?.unwrap();
    m.add("RustWidget", widget_class)?;

    Ok(())
}
