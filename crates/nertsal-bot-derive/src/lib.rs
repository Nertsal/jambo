use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Bot)]
pub fn bot_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let gen = quote! {
        #[async_trait]
        impl Bot for #name {
            fn name(&self) -> &str {
                Self::name()
            }

            async fn handle_server_message(
                &mut self,
                client: &TwitchClient,
                message: &ServerMessage,
            ) {
                match message {
                    ServerMessage::Privmsg(message) => {
                        perform_commands(self,
                            client,
                            self.channel_login.clone(),
                            &private_to_command_message(message),
                        )
                        .await;
                    }
                    _ => (),
                };
            }

            async fn handle_command_message(&mut self,
                client: &TwitchClient,
                message: &CommandMessage<Sender>,
            ) {
                perform_commands(self, client, self.channel_login.clone(), message).await;
            }

            async fn update(
                &mut self,
                client: &TwitchClient,
                delta_time: f32,
            ) {
                self.handle_update(client, delta_time).await;
            }

            fn get_completion_tree(&self) -> Vec<CompletionNode> {
                commands_to_completion(&self.get_commands().commands)
            }
        }
    };

    TokenStream::from(gen)
}
