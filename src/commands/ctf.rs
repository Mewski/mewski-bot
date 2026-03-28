use poise::serenity_prelude::{self as serenity, ChannelFlags, ChannelType};

use crate::{Context, Error};

const CATEGORIES: &[&str] = &[
  "forensics",
  "network",
  "rev",
  "crypto",
  "web",
  "stego",
  "log-analysis",
  "misc",
];

#[poise::command(slash_command, subcommands("create"))]
pub async fn ctf(_ctx: Context<'_>) -> Result<(), Error> {
  Ok(())
}

/// Create a new CTF workspace.
#[poise::command(slash_command, guild_only)]
async fn create(
  ctx: Context<'_>,
  #[description = "Name of the CTF"] name: String,
) -> Result<(), Error> {
  let guild_id = ctx.guild_id().ok_or("Must be used in a server")?;
  let channel = ctx
    .channel_id()
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Not a guild channel")?;

  let category_id = channel.parent_id.ok_or("Run this inside a category")?;

  let forum_name = format!("ctf-{name}");
  let guild = guild_id.to_partial_guild(ctx).await?;
  let duplicate = guild
    .channels(ctx)
    .await?
    .into_values()
    .any(|ch| ch.kind == ChannelType::Forum && ch.name.eq_ignore_ascii_case(&forum_name));

  if duplicate {
    return Err(format!("A CTF named **{name}** already exists").into());
  }

  let mut forum = guild_id
    .create_channel(
      ctx,
      serenity::CreateChannel::new(&forum_name)
        .kind(ChannelType::Forum)
        .category(category_id)
        .position(channel.position + 1),
    )
    .await?;

  let tags: Vec<_> = ["unsolved", "solved"]
    .iter()
    .chain(CATEGORIES.iter())
    .map(|t| serenity::CreateForumTag::new(*t))
    .collect();

  forum
    .edit(ctx, serenity::EditChannel::new().available_tags(tags))
    .await?;

  let general = forum
    .create_forum_post(
      ctx,
      serenity::CreateForumPost::new(
        "General",
        serenity::CreateMessage::new().content(
          "Use `/chal create <name> <category>` to add challenges.\n\
           Solve them with `/chal solve <flag>` in the challenge thread.",
        ),
      ),
    )
    .await?;

  general
    .id
    .edit_thread(ctx, serenity::EditThread::new().flags(ChannelFlags::PINNED))
    .await?;

  ctx.say(format!("Created <#{}>.", forum.id)).await?;
  Ok(())
}
