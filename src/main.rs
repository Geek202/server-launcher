use webhook::Webhook;
use std::error::Error;
use std::process::{Command};
use std::fs;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let command = args[0].to_owned();
    args.remove(0);

    let webhook_url = std::env::var("WEBHOOK_URL").unwrap();
    let webhook = Webhook::from_url(&*webhook_url);

    send_message(&webhook, "Starting server...").await?;

    loop {
        let exit_code = run_server(command.to_owned(), &args).await?;
        if exit_code == 0 {
            let was_restart = std::path::Path::new(".restart_reason").exists();
            if was_restart {
                let restart_reason = fs::read_to_string(".restart_reason").expect("Failed to read restart reason!");
                send_message(&webhook, &*("Restarting server as it was requested!\nReason: ".to_owned() + &restart_reason)).await?;
                fs::remove_file(".restart_reason").expect("failed to remove restart reason file!");
            } else {
                send_message(&webhook, "Server exited normally, not restarting!").await?;
                break;
            }
        } else if exit_code == 0xFFAAFF {
            send_message(&webhook, "Server didn't return an exit code? Assuming hard crash and restarting...").await?;
        } else {
            send_message(&webhook, "Server exited with non-zero exit code, restarting...").await?;
        }
    }

    Ok(())
}

async fn send_message(webhook: &Webhook, msg: &str) -> Result<(), Box<dyn Error>> {
    webhook.send(|message| message.content(msg)).await
}

async fn run_server(program: String, args: &Vec<String>) -> std::io::Result<i32> {
    let code = Command::new(program)
        .args(args)
        .spawn()?.wait()?
        .code().unwrap_or_else(|| { 0xFFAAFF });
    Ok(code)
}
