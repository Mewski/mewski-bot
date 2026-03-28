use poise::serenity_prelude::{CreateMessage, GetMessages, GuildId};
use tokio::process::Command;

use crate::{Context, Error};

const ALLOWED_GUILD: GuildId = GuildId::new(1476649920063868980);
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

/// Summarize recent discoveries and findings in this channel.
#[poise::command(slash_command, guild_only)]
pub async fn summarize(
  ctx: Context<'_>,
  #[description = "Number of messages to look back (default 100, max 500)"]
  #[min = 1]
  #[max = 500]
  messages: Option<u16>,
) -> Result<(), Error> {
  if ctx.guild_id() != Some(ALLOWED_GUILD) {
    return Err("This command is not available in this server".into());
  }

  let count = messages.unwrap_or(100);

  ctx.defer().await?;

  let mut history = Vec::new();
  let mut remaining = count;
  let mut before = None;

  while remaining > 0 {
    let batch_size = remaining.min(100) as u8;
    let mut query = GetMessages::new().limit(batch_size);
    if let Some(id) = before {
      query = query.before(id);
    }

    let batch = ctx.channel_id().messages(ctx, query).await?;
    if batch.is_empty() {
      break;
    }

    before = batch.last().map(|m| m.id);
    remaining = remaining.saturating_sub(batch.len() as u16);
    history.extend(batch);
  }

  if history.is_empty() {
    return Err("No messages found in this channel".into());
  }

  let tmp_dir = tempfile::tempdir()?;
  let http = reqwest::Client::new();
  let channel_name = ctx
    .channel_id()
    .to_channel(ctx)
    .await?
    .guild()
    .map(|c| c.name.clone())
    .unwrap_or_else(|| "unknown".into());

  let mut prompt = format!(
    "Summarize the key discoveries, findings, and progress from this CTF channel log. \
     Be concise and use bullet points. Focus on technical findings, solved steps, \
     identified vulnerabilities, useful artifacts, and any open leads. Ignore casual chatter.\n\n\
     Channel: #{channel_name}\n\n"
  );

  let mut has_content = false;

  for msg in history.iter().rev() {
    if !msg.content.is_empty() {
      prompt.push_str(&format!("{}: {}\n", msg.author.name, msg.content));
      has_content = true;
    }

    for attachment in &msg.attachments {
      let ext = attachment
        .filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

      if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        continue;
      }

      let Ok(resp) = http.get(&attachment.url).send().await else {
        continue;
      };
      let Ok(bytes) = resp.bytes().await else {
        continue;
      };

      let path = tmp_dir.path().join(&attachment.filename);
      if tokio::fs::write(&path, &bytes).await.is_ok() {
        prompt.push_str(&format!(
          "{} shared image: read and analyze {}\n",
          msg.author.name,
          path.display()
        ));
        has_content = true;
      }
    }
  }

  if !has_content {
    return Err("No messages or images found to summarize".into());
  }

  let mut child = Command::new("claude")
    .args([
      "--print",
      "--model",
      "claude-opus-4-6",
      "--allowedTools",
      "Read",
    ])
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .map_err(|e| format!("Failed to run claude: {e}"))?;

  if let Some(mut stdin) = child.stdin.take() {
    use tokio::io::AsyncWriteExt;
    stdin.write_all(prompt.as_bytes()).await?;
  }

  let output = child.wait_with_output().await?;

  drop(tmp_dir);

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("claude exited with error: {stderr}").into());
  }

  let summary = String::from_utf8_lossy(&output.stdout);
  let summary = summary.trim();

  if summary.is_empty() {
    return Err("Claude returned an empty response".into());
  }

  for chunk in chunk_str(summary, 2000) {
    ctx
      .channel_id()
      .send_message(ctx, CreateMessage::new().content(chunk))
      .await?;
  }

  Ok(())
}

fn chunk_str(s: &str, max: usize) -> Vec<&str> {
  let mut chunks = Vec::new();
  let mut start = 0;
  while start < s.len() {
    let end = (start + max).min(s.len());
    let end = if end < s.len() {
      s[start..end]
        .rfind('\n')
        .map(|i| start + i + 1)
        .unwrap_or(end)
    } else {
      end
    };
    chunks.push(&s[start..end]);
    start = end;
  }
  chunks
}
