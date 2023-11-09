use dotenv::dotenv;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "check actual gas price in gwei")]
    Gas,
    #[command(description = "set alert for gas price in gwei")]
    GasAlert(u64),
    #[command(description = "remove all alerts")]
    ClearAlerts,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Gas => {
            bot.send_message(msg.chat.id, format!("The gas is '10' gwei"))
                .await?
        }
        Command::GasAlert(gas) => {
            bot.send_message(
                msg.chat.id,
                format!("Your gas request for {gas} gwei was set!"),
            )
            .await?
        }
        Command::ClearAlerts => {
            bot.send_message(msg.chat.id, format!("Alerts cleared"))
                .await?
        }
    };

    Ok(())
}
