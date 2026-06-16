use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChatPanelProps {
    pub agent_id: Option<String>,
}

#[function_component(ChatPanel)]
pub fn chat_panel(_props: &ChatPanelProps) -> Html {
    html! {
        <div class="chat-panel-placeholder">
            <p>{"Chat panel is being rebuilt for lesson conversations."}</p>
        </div>
    }
}
