use ethers::{
    providers::{Http, Middleware},
    types::U256,
};
use std::env;
use std::fs;
use std::time::Duration;
use telegram_bot::*;
use tokio;

#[derive(Debug, PartialEq)]
enum Command {
    CheckGas,
    SetGasAlert(u64), // Takes a threshold in gwei as an argument
}

async fn get_gas_price() -> Result<U256, ethers::Error> {
    let infura_api_key =
        env::var("INFURA_API_KEY").expect("INFURA_API_KEY not set in environment variables");

    // Connect to an Ethereum node (e.g., Infura)
    let http = Http::connect(&format!("https://mainnet.infura.io/v3/{}", infura_api_key)).await?;
    let provider = http.into();

    // Get the gas price from the blockchain (in wei)
    let gas_price = provider.get_gas_price().await?;
    Ok(gas_price)
}

async fn send_gas_alert(
    api: Api,
    chat_id: ChatId,
    threshold: U256,
) -> Result<(), Box<dyn std::error::Error>> {
    let gas_price_gwei = threshold / U256::exp10(9);
    api.send(chat_id.text(format!("Gas price is below {} gwei!", gas_price_gwei)))
        .await?;
    Ok(())
}

fn read_thresholds(path: &str) -> Vec<U256> {
    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(thresholds) = serde_json::from_str(&contents) {
            return thresholds;
        }
    }
    vec![]
}

fn write_thresholds(path: &str, thresholds: Vec<U256>) -> std::io::Result<()> {
    let json_content = serde_json::to_string(&thresholds)?;
    fs::write(path, json_content)
}

async fn handle_message(
    api: Api,
    message: Message,
    command: Command,
    persisted_data_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::CheckGas => {
            let gas_price_wei = get_gas_price().await?;
            let gas_price_gwei = gas_price_wei / U256::exp10(9);
            api.send(
                message
                    .chat
                    .text(format!("Current gas price: {} gwei", gas_price_gwei)),
            )
            .await?;
        }
        Command::SetGasAlert(threshold_gwei) => {
            let mut thresholds = read_thresholds(persisted_data_path);

            let threshold_wei = U256::from(threshold_gwei) * U256::exp10(9); // Convert gwei to wei

            thresholds.push(threshold_wei); // Add the new threshold

            write_thresholds(persisted_data_path, thresholds.clone())?;

            api.send(message.chat.text(format!(
                "Gas alert threshold set to {} gwei",
                threshold_gwei
            )))
            .await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_token = env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN not set in environment variables");
    let chat_id =
        ChatId::new(env::var("CHAT_ID").expect("CHAT_ID not set in environment variables"));
    let persisted_data_path = env::var("PERSISTED_DATA_PATH")
        .expect("PERSISTED_DATA_PATH not set in environment variables");

    let api = Api::new(api_token);
    let mut thresholds = read_thresholds(&persisted_data_path);

    if thresholds.is_empty() {
        thresholds.push(U256::from(100_000_000)); // Default threshold if none are set (in wei)
        write_thresholds(&persisted_data_path, thresholds.clone())?;
    }

    let updates = api.stream().await?;
    for update in updates {
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                let command = match data {
                    "/checkgas" => Command::CheckGas,
                    s if s.starts_with("/setgasalert ") => {
                        if let Some(threshold) = s.split_whitespace().nth(1) {
                            if let Ok(threshold) = threshold.parse::<u64>() {
                                Command::SetGasAlert(threshold)
                            } else {
                                continue; // Invalid threshold, ignore the message
                            }
                        } else {
                            continue; // No threshold provided, ignore the message
                        }
                    }
                    _ => continue,
                };
                handle_message(api.clone(), message, command, &persisted_data_path).await?;
            }
        }
    }

    Ok(())
}
