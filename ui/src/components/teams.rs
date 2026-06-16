use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TeamsProps {
    pub auth_token: String,
}

#[function_component(Teams)]
pub fn teams_component(_props: &TeamsProps) -> Html {
    html! {
        <div class="teams-placeholder">
            <h2>{"Teams"}</h2>
            <p>{"Team management has been deprecated. Use the student dashboard or admin panel instead."}</p>
        </div>
    }
}
