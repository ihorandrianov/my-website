mod handlers;
mod keyboard;
mod responses;

use std::sync::Arc;

use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateFilterExt},
    dptree,
    prelude::*,
    update_listeners::webhooks::{self, Options},
};

pub use handlers::{BotState, Command, State};

pub async fn init_bot(
    bot: Arc<Bot>,
    webhook_secret: String,
    bot_state: BotState,
) -> anyhow::Result<axum::Router> {
    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(dptree::case![State::Unauthorized].endpoint(handlers::handle_unauthorized))
        .branch(
            dptree::case![State::Authorized]
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(handlers::handle_command),
                )
                .branch(dptree::endpoint(handlers::handle_message)),
        );

    let callback_handler = Update::filter_callback_query().endpoint(handlers::handle_callback);

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);

    let (listener, _stop_flag, router) = webhooks::axum_to_router(
        (*bot).clone(),
        Options::new(
            "0.0.0.0:6060".parse().unwrap(),
            "https://andrianov.dev/api/webhook-tg".parse().unwrap(),
        )
        .secret_token(webhook_secret),
    )
    .await?;

    let bot_for_dispatcher = (*bot).clone();
    tokio::spawn(async move {
        Dispatcher::builder(bot_for_dispatcher, handler)
            .dependencies(dptree::deps![InMemStorage::<State>::new(), bot_state])
            .build()
            .dispatch_with_listener(
                listener,
                LoggingErrorHandler::with_custom_text("Error from the update listener"),
            )
            .await;
    });

    Ok(router)
}
