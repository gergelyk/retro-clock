use leptos::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn InfoDialog(
    set_show_popup: WriteSignal<Option<String>>,
    content: String,
    popup_button_ref: NodeRef<leptos::html::Button>,
) -> impl IntoView {
    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
            <div class="bg-white rounded-lg p-6 max-w-sm shadow-lg relative">
                // This would be better to pass view instead of raw HTML here
                <p class="mb-4 text-left">
                    <div inner_html=content />
                </p>
                <p class="mb-4 text-right">
                    <button
                        node_ref=popup_button_ref
                        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                        on:click=move |_| set_show_popup.set(None)
                    >
                        "Okay"
                    </button>
                </p>
            </div>
        </div>
    }
}
