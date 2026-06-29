use jupyter_counter_types::{CounterBackendMessage, CounterFrontendMessage};
use jupyter_rust_widget_frontend::{AnyWidgetModel, Context, WasmWidget};
use wasm_bindgen::{
    JsCast, JsValue,
    prelude::{Closure, wasm_bindgen},
};
use web_sys::{Element, HtmlButtonElement};

struct Counter {
    button: Option<HtmlButtonElement>,
}

impl Counter {
    fn new() -> Self {
        Self { button: None }
    }
}

impl WasmWidget for Counter {
    type BackendMessage = CounterBackendMessage;
    type FrontendMessage = CounterFrontendMessage;

    fn handle_message(&mut self, message: Self::FrontendMessage) {
        match message {
            CounterFrontendMessage::NewValue { value } => {
                if let Some(button) = &self.button {
                    button.set_inner_html(&format!("count is {}", value));
                }
            }
        }
    }

    fn mount(&mut self, context: Context<CounterBackendMessage>, element: Element) {
        let document = web_sys::window().unwrap().document().unwrap();
        let button = document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();

        button.set_inner_html("count is 0");

        let model_clone = context.clone();
        let click_closure = Closure::<dyn FnMut()>::new(move || {
            model_clone.send_message(&CounterBackendMessage::Increment);
        });

        button
            .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
            .unwrap();
        click_closure.forget();

        element.append_child(&button).unwrap();
        self.button = Some(button);
    }
}

#[wasm_bindgen]
pub fn render_counter_impl(model: AnyWidgetModel, el: Element) -> Result<(), JsValue> {
    let counter = Counter::new();
    counter.render(model, el)
}
