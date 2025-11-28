mod commands;

use poise::serenity_prelude as serenity;

pub struct Data {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
  pretty_env_logger::formatted_builder().init();

  dotenvy::dotenv().ok();

  let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
  let intents = serenity::GatewayIntents::non_privileged();

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![commands::ctf(), commands::chal()],
      ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        Ok(Data {})
      })
    })
    .build();

  let client = serenity::ClientBuilder::new(token, intents)
    .framework(framework)
    .await;

  client?.start().await?;

  Ok(())
}
