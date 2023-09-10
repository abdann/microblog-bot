use crate::{json_models::Post, Context, Error};
use log::info;

/// Generates a log of a post.
async fn log_post(title: &str, body: &str, ctx: Context<'_>) -> String {
    // Log the post itself alongside the author.
    let mut log = format!(
        "Post with title '{}' and body '{}'. Requested by '{}' with ID '{}'.",
        &title,
        &body,
        ctx.author().name,
        ctx.author().id.0
    );
    // Log the guild that the post occurred in, if possible.
    if let Some(guild) = ctx.partial_guild().await {
        let name = guild.name;
        let id = guild.id.0;
        let addendum = format!(" In server '{}' with ID '{}'.", name, id);
        log += &addendum;
    }
    log
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            // extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    crate::teardown(
        ctx.framework().shard_manager,
        format!(
            "User '{}' with ID '{}' used the '{}' command.",
            ctx.author().name,
            ctx.author().id.0,
            ctx.command().name
        )
        .as_str(),
    )
    .await;
    Ok(())
}

// Post to the microblog
#[poise::command(prefix_command, slash_command)]
pub async fn microblog(
    ctx: Context<'_>,
    #[description = "The title for the blog post"] title: String,
    #[description = "The body of the blog post"] body: String,
) -> Result<(), Error> {
    let tag = ctx.author().name.clone();
    let post = Post {
        title: &title,
        tag: &tag,
        body: &body,
    };
    {
        let logged_in = *ctx.data().logged_in.lock().await;
        if !logged_in {
            login(ctx).await?;
        }
    }
    let csrf_token = ctx.data().csrf_token.lock().await;
    let _response = ctx
        .data()
        .client
        .post(&ctx.data().post_endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("X-CSRFToken", csrf_token.as_ref().unwrap())
        .header(reqwest::header::REFERER, &ctx.data().domain)
        .json(&post)
        .send()
        .await?;
    let log = log_post(&title, &body, ctx).await;
    info!(target: "post-logger", "{}", &log);
    reply_with_post(ctx, post).await?;
    Ok(())
}

pub async fn reply_with_post(ctx: Context<'_>, post: Post<'_>) -> Result<(), Error> {
    poise::send_reply(ctx, |f| {
        f.embed(|f| {
            f.author(|f| f.name(post.tag))
                .description(post.body)
                .title(post.title)
        })
        .content("Blog post created!")
    })
    .await?;
    Ok(())
}

/// Login the bot
pub async fn login(ctx: Context<'_>) -> Result<(), Error> {
    let login = &ctx.data().credentials;
    let response = ctx
        .data()
        .client
        .post(&ctx.data().login_endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&login)
        .send()
        .await?;
    let csrftoken_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == "csrftoken")
        .unwrap();
    let csrftoken = csrftoken_cookie.value();
    {
        let mut logged_in = ctx.data().logged_in.lock().await;
        *logged_in = true;
    }
    {
        let mut csrftoken_stored = ctx.data().csrf_token.lock().await;
        *csrftoken_stored = Some(csrftoken.to_owned());
    }
    info!(target: "login-logger", "Logged in.");
    Ok(())
}
