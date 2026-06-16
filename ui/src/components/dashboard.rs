use yew::prelude::*;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    html! {
        <div class="dashboard-placeholder">
            <h2>{"Dashboard"}</h2>
            <p>{"The legacy agent dashboard has been deprecated. Use the student dashboard or admin panel instead."}</p>
        </div>
    }
}
