use jupyter_counter_types::{CounterBackendMessage, CounterFrontendMessage};
use jupyter_rust_widget_backend::{RustWidget, inject_rust_widget};
use pyo3::{
    Bound, PyAny, PyResult, Python, pyfunction, pymodule,
    types::{PyModule, PyModuleMethods as _},
    wrap_pyfunction,
};

struct Counter {
    count: u64,
}

impl RustWidget for Counter {
    type BackendMessage = CounterBackendMessage;
    type FrontendMessage = CounterFrontendMessage;

    fn esm(&self) -> String {
        let bundle = include_str!("../../jupyter_counter_frontend/dist/bundle.js");
        let mut esm = String::from(bundle);
        esm.push_str(
            r#" async function render({model, el}) {
                  await JupyterCounter.render_counter(model, el);
            }
            export default { render }"#,
        );
        esm
    }

    fn handle_message(&mut self, event: Self::BackendMessage) -> Option<Self::FrontendMessage> {
        match event {
            CounterBackendMessage::Increment => {
                self.count += 1;
                Some(CounterFrontendMessage::NewValue { value: self.count })
            }
        }
    }
}

#[pyfunction(name = "Counter")]
fn make_counter(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    Counter { count: 0 }.into_widget(py, "jupyter_counter")
}

#[pymodule]
fn jupyter_counter(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    inject_rust_widget(py, m)?;
    m.add_function(wrap_pyfunction!(make_counter, m)?)?;
    Ok(())
}
