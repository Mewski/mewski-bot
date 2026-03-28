use poise::serenity_prelude::{self as serenity, ChannelType, GuildChannel};

use crate::{Context, Error};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum Category {
  #[name = "forensics"]
  Forensics,
  #[name = "network"]
  Network,
  #[name = "rev"]
  Rev,
  #[name = "crypto"]
  Crypto,
  #[name = "web"]
  Web,
  #[name = "stego"]
  Stego,
  #[name = "log-analysis"]
  LogAnalysis,
  #[name = "misc"]
  Misc,
}

impl Category {
  fn as_str(&self) -> &'static str {
    match self {
      Self::Forensics => "forensics",
      Self::Network => "network",
      Self::Rev => "rev",
      Self::Crypto => "crypto",
      Self::Web => "web",
      Self::Stego => "stego",
      Self::LogAnalysis => "log-analysis",
      Self::Misc => "misc",
    }
  }
}

/// Resolve the parent forum of a thread, validating it's a CTF forum.
async fn resolve_ctf_forum(ctx: Context<'_>, thread: &GuildChannel) -> Result<GuildChannel, Error> {
  let parent_id = thread
    .parent_id
    .ok_or("This thread has no parent channel")?;
  let forum = parent_id
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Parent channel not found")?;

  if forum.kind != ChannelType::Forum || !forum.name.starts_with("ctf-") {
    return Err("This command must be used inside a CTF forum".into());
  }

  Ok(forum)
}

fn find_tag_id(forum: &GuildChannel, name: &str) -> Option<serenity::ForumTagId> {
  forum
    .available_tags
    .iter()
    .find(|t| t.name.eq_ignore_ascii_case(name))
    .map(|t| t.id)
}

#[poise::command(slash_command, subcommands("create", "solve"))]
pub async fn chal(_ctx: Context<'_>) -> Result<(), Error> {
  Ok(())
}

/// Add a challenge to the current CTF.
#[poise::command(slash_command, guild_only)]
async fn create(
  ctx: Context<'_>,
  #[description = "Challenge name"] name: String,
  #[description = "Challenge category"] category: Category,
) -> Result<(), Error> {
  let thread = ctx
    .channel_id()
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Not a guild channel")?;

  if thread.kind != ChannelType::PublicThread
    || !thread.name.eq_ignore_ascii_case("general")
    || !thread.applied_tags.is_empty()
  {
    return Err("Run this in the CTF's **General** thread".into());
  }

  let forum = resolve_ctf_forum(ctx, &thread).await?;

  if name.starts_with('\u{2714}') {
    return Err("Challenge names cannot start with a checkmark".into());
  }

  let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
  let active = guild_id.get_active_threads(ctx).await?;
  let duplicate = active
    .threads
    .iter()
    .any(|t| t.parent_id == Some(forum.id) && t.name.eq_ignore_ascii_case(&name));

  if duplicate {
    return Err(format!("Challenge **{name}** already exists").into());
  }

  let mut tags = Vec::new();
  if let Some(id) = find_tag_id(&forum, category.as_str()) {
    tags.push(id);
  }
  if let Some(id) = find_tag_id(&forum, "unsolved") {
    tags.push(id);
  }

  let post = forum
    .id
    .create_forum_post(
      ctx,
      serenity::CreateForumPost::new(&name, serenity::CreateMessage::new().content(&name))
        .set_applied_tags(tags),
    )
    .await?;

  ctx
    .say(format!("{} created <#{}>", ctx.author(), post.id))
    .await?;
  Ok(())
}

/// Mark the current challenge as solved.
#[poise::command(slash_command, guild_only)]
async fn solve(ctx: Context<'_>, #[description = "The flag"] flag: String) -> Result<(), Error> {
  let thread = ctx
    .channel_id()
    .to_channel(ctx)
    .await?
    .guild()
    .ok_or("Not a guild channel")?;

  if thread.kind != ChannelType::PublicThread || thread.applied_tags.is_empty() {
    return Err("Run this inside a challenge thread".into());
  }

  let forum = resolve_ctf_forum(ctx, &thread).await?;

  let solved_id = find_tag_id(&forum, "solved");
  if let Some(id) = solved_id {
    if thread.applied_tags.contains(&id) {
      return Err("This challenge is already solved".into());
    }
  }

  let unsolved_id = find_tag_id(&forum, "unsolved");
  let mut new_tags: Vec<_> = thread
    .applied_tags
    .iter()
    .copied()
    .filter(|id| Some(*id) != unsolved_id)
    .collect();

  if let Some(id) = solved_id {
    new_tags.push(id);
  }

  ctx.say("Marking as solved.").await?;

  ctx
    .channel_id()
    .edit_thread(
      ctx,
      serenity::EditThread::new()
        .name(format!("\u{2714}-{}", thread.name))
        .applied_tags(new_tags)
        .archived(true),
    )
    .await?;

  let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
  let active = guild_id.get_active_threads(ctx).await?;
  let general = active
    .threads
    .iter()
    .find(|t| t.parent_id == Some(forum.id) && t.name.eq_ignore_ascii_case("general"));

  if let Some(general) = general {
    general
      .id
      .send_message(
        ctx,
        serenity::CreateMessage::new().content(format!(
          "{} solved **{}** with ||`{}`||",
          ctx.author(),
          thread.name,
          flag.trim(),
        )),
      )
      .await?;
  }

  Ok(())
}
