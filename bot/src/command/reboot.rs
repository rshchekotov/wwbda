use crate::{Context, Error};

#[poise::command(prefix_command, slash_command, owners_only, ephemeral)]
pub async fn reboot(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Rebooting...")
        .await
        .expect("Reboot command should succeed.");

    tokio::fs::File::create("logs/.reboot")
        .await
        .expect("Should be able to set the reboot flag");

    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}
