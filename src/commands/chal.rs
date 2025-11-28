use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, ChannelType};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum Category {
  #[name = "pwn"]
  Pwn,
  #[name = "rev"]
  Rev,
  #[name = "osint"]
  Osint,
  #[name = "crypto"]
  Crypto,
  #[name = "web"]
  Web,
  #[name = "misc"]
  Misc,
  #[name = "forensics"]
  Forensics,
}

impl Category {
  pub fn as_str(&self) -> &'static str {
    match self {
      Category::Pwn => "pwn",
      Category::Rev => "rev",
      Category::Osint => "osint",
      Category::Crypto => "crypto",
      Category::Web => "web",
      Category::Misc => "misc",
      Category::Forensics => "forensics",
    }
  }
}

#[poise::command(slash_command, subcommands("create", "solve"))]
pub async fn chal(_ctx: Context<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn create(
  ctx: Context<'_>,
  #[description = "Name of the challenge"] name: String,
  #[description = "Category of the challenge"] category: Category,
) -> Result<(), Error> {
  let channel = ctx.channel_id();

  let current_channel = channel
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Not a guild channel")?;

  if current_channel.kind != ChannelType::PublicThread
    || current_channel.name.to_lowercase() != "general"
    || !current_channel.applied_tags.is_empty()
  {
    return Err("Must be used inside a CTF forum's general channel.".into());
  }

  let parent_id = current_channel.parent_id.ok_or("Thread has no parent")?;
  let forum = parent_id
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Parent not found")?;

  if forum.kind != ChannelType::Forum || !forum.name.to_lowercase().starts_with("ctf-") {
    return Err("Must be used inside a CTF forum's general channel.".into());
  }

  let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
  let threads = guild_id.get_active_threads(ctx).await?;
  let existing = threads.threads.iter().find(|thread| {
    thread.parent_id == Some(forum.id) && thread.name.to_lowercase() == name.to_lowercase()
  });

  if let Some(existing) = existing {
    return Err(format!("Challenge <#{}> already exists.", existing.id).into());
  }

  let category_tag = forum
    .available_tags
    .iter()
    .find(|tag| tag.name.to_lowercase() == category.as_str())
    .map(|tag| tag.id);

  let unsolved_tag = forum
    .available_tags
    .iter()
    .find(|tag| tag.name.to_lowercase() == "unsolved")
    .map(|tag| tag.id);

  let mut applied_tags = Vec::new();
  if let Some(tag_id) = category_tag {
    applied_tags.push(tag_id);
  }
  if let Some(tag_id) = unsolved_tag {
    applied_tags.push(tag_id);
  }

  let thread = forum
    .id
    .create_forum_post(
      ctx,
      serenity::CreateForumPost::new(&name, serenity::CreateMessage::new().content(&name))
        .set_applied_tags(applied_tags),
    )
    .await?;

  ctx
    .say(format!("{} created <#{}>.", ctx.author(), thread.id))
    .await?;

  Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn solve(
  ctx: Context<'_>,
  #[description = "The flag for the challenge"] flag: String,
) -> Result<(), Error> {
  let channel_id = ctx.channel_id();
  let thread = channel_id
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Not a guild channel")?;

  if thread.kind != ChannelType::PublicThread || thread.applied_tags.is_empty() {
    return Err("Must be used inside a challenge channel.".into());
  }

  let parent_id = thread.parent_id.ok_or("Thread has no parent")?;
  let forum = parent_id
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Parent not found")?;

  if forum.kind != ChannelType::Forum || !forum.name.to_lowercase().starts_with("ctf-") {
    return Err("Must be used inside a challenge channel.".into());
  }

  let solved_tag = forum
    .available_tags
    .iter()
    .find(|tag| tag.name.to_lowercase() == "solved")
    .map(|tag| tag.id);

  if let Some(tag_id) = solved_tag {
    if thread.applied_tags.contains(&tag_id) {
      return Err("This challenge is already solved.".into());
    }
  }

  let unsolved_tag = forum
    .available_tags
    .iter()
    .find(|tag| tag.name.to_lowercase() == "unsolved")
    .map(|tag| tag.id);

  let mut new_tags: Vec<_> = thread
    .applied_tags
    .iter()
    .filter(|&&tag_id| Some(tag_id) != unsolved_tag)
    .copied()
    .collect();

  if let Some(tag_id) = solved_tag {
    if !new_tags.contains(&tag_id) {
      new_tags.push(tag_id);
    }
  }

  let new_name = format!("\u{2714}-{}", thread.name);

  ctx.say("Marking challenge as solved.").await?;

  channel_id
    .edit_thread(
      ctx,
      serenity::EditThread::new()
        .name(&new_name)
        .applied_tags(new_tags)
        .archived(true),
    )
    .await?;

  let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
  let threads = guild_id.get_active_threads(ctx).await?;

  let general = threads
    .threads
    .iter()
    .find(|thread| thread.parent_id == Some(parent_id) && thread.name.to_lowercase() == "general");

  if let Some(general) = general {
    general
      .id
      .send_message(
        ctx,
        serenity::CreateMessage::new().content(format!(
          "{} solved challenge <#{}> with ||{}||",
          ctx.author(),
          channel_id,
          flag
        )),
      )
      .await?;
  }

  Ok(())
}
