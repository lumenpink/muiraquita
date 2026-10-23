use dioxus::prelude::*;

#[component]
pub fn AuthModal(on_login: EventHandler<String>, on_close: EventHandler<()>) -> Element {
    let mut domain = use_signal(|| "https://lumen.pink/".to_string());

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-dialog",
                header { class: "modal-header",
                    h3 { "🔑 Login IndieAuth" }
                    button {
                        class: "modal-close-btn",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }
                div { class: "modal-body",
                    p { "Digite a URL do seu domínio ou blog pessoal IndieWeb:" }
                    input {
                        class: "input-field",
                        r#type: "url",
                        placeholder: "https://seu-dominio.com/",
                        value: "{domain}",
                        oninput: move |e| domain.set(e.value()),
                    }
                    p { class: "modal-help-text",
                        "O Muiraquitã descobrirá seus endpoints de autorização, Micropub e Microsub conforme a especificação oficial W3C."
                    }
                }
                footer { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Cancelar"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| on_login.call(domain.read().clone()),
                        "Conectar Perfil"
                    }
                }
            }
        }
    }
}
