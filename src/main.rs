use webhook::Webhook;
use std::error::Error;
use std::process::{Command};
use std::fs;
use std::time::{Instant, Duration};


const RESTART_INTERVAL: Duration = Duration::from_secs(4 * 60);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let command = args[0].to_owned();
    args.remove(0);

    let webhook_url = std::env::var("WEBHOOK_URL").unwrap_or_else(|_e| { String::from("") });
    let webhook = if webhook_url != "" {
        Some(Webhook::from_url(&*webhook_url))
    } else {
        None
    };

    send_message(&webhook, "Starting server...").await?;

    loop {
        let start_time = Instant::now();
        let exit_code = run_server(command.to_owned(), &args).await?;
        let exit_time = Instant::now();
        let runtime = exit_time - start_time;

        let too_fast = runtime < RESTART_INTERVAL;

        if exit_code == 0 {
            let was_restart = std::path::Path::new(".restart_reason").exists();
            if was_restart {
                if too_fast {
                    let pause_time = RESTART_INTERVAL - runtime;
                    send_message(&webhook, &*format!("Server exited after {:?}s! Waiting {:?}s before restarting...", runtime.as_secs(), pause_time.as_secs())).await?;
                    tokio::time::delay_for(pause_time).await;
                }

                let restart_reason = fs::read_to_string(".restart_reason").expect("Failed to read restart reason!");
                send_message(&webhook, &*("Restarting server as it was requested!\nReason: ".to_owned() + &restart_reason)).await?;
                fs::remove_file(".restart_reason").expect("failed to remove restart reason file!");
            } else {
                send_message(&webhook, "Server exited normally, not restarting!").await?;
                break;
            }
        } else if exit_code == 0xFFAAFF {
            if too_fast {
                let pause_time = RESTART_INTERVAL - runtime;
                send_message(&webhook, &*format!("Server exited after {:?}s! Waiting {:?}s before restarting...", runtime.as_secs(), pause_time.as_secs())).await?;
                tokio::time::delay_for(pause_time).await;
            }

            send_message(&webhook, "Server didn't return an exit code? Assuming hard crash and restarting...").await?;
        } else {
            if too_fast {
                let pause_time = RESTART_INTERVAL - runtime;
                send_message(&webhook, &*format!("Server exited after {:?}s! Waiting {:?}s before restarting...", runtime.as_secs(), pause_time.as_secs())).await?;
                tokio::time::delay_for(pause_time).await;
            }

            send_message(&webhook, "Server exited with non-zero exit code, restarting...").await?;
        }
    }

    Ok(())
}

async fn send_message(webhook: &Option<Webhook>, msg: &str) -> Result<(), Box<dyn Error>> {
    match webhook {
        Some(webhook) => webhook.send(|message| message.content(msg)).await,
        _ => Ok(())
    }
}

async fn run_server(program: String, args: &Vec<String>) -> std::io::Result<i32> {
    let code = Command::new(program)
        .args(args)
        .spawn()?.wait()?
        .code().unwrap_or_else(|| { 0xFFAAFF });
    Ok(code)
}
