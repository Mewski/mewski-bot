use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, ChannelFlags, ChannelType};

#[poise::command(slash_command, subcommands("create"))]
pub async fn ctf(_ctx: Context<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn create(
  ctx: Context<'_>,
  #[description = "Name of the CTF"] name: String,
) -> Result<(), Error> {
  let guild_id = ctx.guild_id().ok_or("Must be used in a server")?;
  let channel = ctx.channel_id();

  let parent_category = channel
    .to_channel(ctx)
    .await?
    .guild()
    .and_then(|channel| channel.parent_id)
    .ok_or("Must be used in a category")?;

  let guild = guild_id.to_partial_guild(ctx).await?;
  let existing = guild.channels(ctx).await?.into_iter().find(|(_, channel)| {
    channel.kind == ChannelType::Forum
      && channel.parent_id == Some(parent_category)
      && channel.name.to_lowercase() == name.to_lowercase()
  });

  if let Some((_, existing)) = existing {
    return Err(format!("CTF <#{}> already exists.", existing.id).into());
  }

  let mut forum = guild_id
    .create_channel(
      ctx,
      serenity::CreateChannel::new(&name)
        .kind(ChannelType::Forum)
        .category(parent_category),
    )
    .await?;

  let tags = vec![
    serenity::CreateForumTag::new("pwn"),
    serenity::CreateForumTag::new("rev"),
    serenity::CreateForumTag::new("osint"),
    serenity::CreateForumTag::new("crypto"),
    serenity::CreateForumTag::new("web"),
    serenity::CreateForumTag::new("solved"),
    serenity::CreateForumTag::new("unsolved"),
  ];

  forum
    .edit(ctx, serenity::EditChannel::new().available_tags(tags))
    .await?;

  let general_thread = forum
    .create_forum_post(
      ctx,
      serenity::CreateForumPost::new(
        "General",
        serenity::CreateMessage::new().content(
          "To get started, run `/chal create <name> <category>`\n\
           Solve a challenge in its respective channel with `/chal solve <flag>`",
        ),
      ),
    )
    .await?;

  general_thread
    .id
    .edit_thread(ctx, serenity::EditThread::new().flags(ChannelFlags::PINNED))
    .await?;

  ctx.say(format!("Created <#{}>.", forum.id)).await?;

  Ok(())
}
