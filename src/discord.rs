use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RpcStatus {
    Disabled,
    Connecting,
    Connected,
    WaitingDiscord,
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RpcEvent {
    SetEnabled(bool),
    SetClientId(String),
    InMenu(usize),
    Playing(String, String),
    Reconnect,
}

pub fn spawn_discord_rpc_worker(
    rx: mpsc::Receiver<RpcEvent>,
    status_tx: mpsc::Sender<RpcStatus>,
    initial_enabled: bool,
    initial_client_id: String,
) {
    thread::spawn(move || {
        let mut enabled = initial_enabled;
        let mut client_id = initial_client_id;
        let mut client: Option<DiscordIpcClient> = None;
        let mut last_state: Option<RpcEvent> = None;
        let mut last_connect_attempt = Instant::now() - Duration::from_secs(10);

        if !enabled {
            let _ = status_tx.send(RpcStatus::Disabled);
        } else {
            let _ = status_tx.send(RpcStatus::Connecting);
        }

        let send_activity = |c: &mut Option<DiscordIpcClient>,
                             tx: &mpsc::Sender<RpcStatus>,
                             evt: &RpcEvent| {
            if let Some(ipc) = c.as_mut() {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                let res = match evt {
                    RpcEvent::InMenu(count) => {
                        let state_str = format!("{} jeux dans la bibliothèque", count);
                        let act = activity::Activity::new()
                            .details("Navigue dans sa bibliothèque")
                            .state(&state_str)
                            .timestamps(activity::Timestamps::new().start(ts));
                        ipc.set_activity(act)
                    }
                    RpcEvent::Playing(name, platform) => {
                        let details_str = format!("Joue à {}", name);
                        let state_str = format!("Via {} • UniLauncher", platform);
                        let act = activity::Activity::new()
                            .details(&details_str)
                            .state(&state_str)
                            .timestamps(activity::Timestamps::new().start(ts));
                        ipc.set_activity(act)
                    }
                    _ => Ok(()),
                };

                if let Err(e) = res {
                    let _ = ipc.close();
                    *c = None;
                    let err_msg = e.to_string();
                    if err_msg.contains("10002") || err_msg.contains("Unknown Application") {
                        let _ = tx.send(RpcStatus::Error("ID Discord invalide (Erreur 10002)".into()));
                    } else {
                        let _ = tx.send(RpcStatus::WaitingDiscord);
                    }
                } else {
                    let _ = tx.send(RpcStatus::Connected);
                }
            }
        };

        loop {
            match rx.recv_timeout(Duration::from_millis(600)) {
                Ok(evt) => match evt {
                    RpcEvent::SetEnabled(on) => {
                        enabled = on;
                        if !enabled {
                            if let Some(mut ipc) = client.take() {
                                let _ = ipc.clear_activity();
                                let _ = ipc.close();
                            }
                            let _ = status_tx.send(RpcStatus::Disabled);
                        } else {
                            let _ = status_tx.send(RpcStatus::Connecting);
                            last_connect_attempt = Instant::now() - Duration::from_secs(10);
                        }
                    }
                    RpcEvent::SetClientId(new_id) => {
                        if client_id != new_id {
                            client_id = new_id;
                            if let Some(mut ipc) = client.take() {
                                let _ = ipc.clear_activity();
                                let _ = ipc.close();
                            }
                            if enabled {
                                let _ = status_tx.send(RpcStatus::Connecting);
                                last_connect_attempt = Instant::now() - Duration::from_secs(10);
                            }
                        }
                    }
                    RpcEvent::Reconnect => {
                        if let Some(mut ipc) = client.take() {
                            let _ = ipc.clear_activity();
                            let _ = ipc.close();
                        }
                        if enabled {
                            let _ = status_tx.send(RpcStatus::Connecting);
                            last_connect_attempt = Instant::now() - Duration::from_secs(10);
                        }
                    }
                    RpcEvent::InMenu(count) => {
                        last_state = Some(RpcEvent::InMenu(count));
                        if enabled && client.is_some() {
                            send_activity(&mut client, &status_tx, last_state.as_ref().unwrap());
                        }
                    }
                    RpcEvent::Playing(name, platform) => {
                        last_state = Some(RpcEvent::Playing(name, platform));
                        if enabled && client.is_some() {
                            send_activity(&mut client, &status_tx, last_state.as_ref().unwrap());
                        }
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if enabled && client.is_none() {
                let trimmed = client_id.trim();
                if trimmed.is_empty() {
                    let _ = status_tx.send(RpcStatus::Error("Client ID Discord non renseigné".into()));
                } else if last_connect_attempt.elapsed() >= Duration::from_secs(3) {
                    last_connect_attempt = Instant::now();
                    let mut ipc = DiscordIpcClient::new(trimmed);
                    match ipc.connect() {
                        Ok(()) => {
                            client = Some(ipc);
                            let _ = status_tx.send(RpcStatus::Connected);
                            if let Some(ref st) = last_state {
                                send_activity(&mut client, &status_tx, st);
                            }
                        }
                        Err(err) => {
                            let err_str = err.to_string();
                            if err_str.contains("10002") || err_str.contains("Unknown Application") {
                                let _ = status_tx.send(RpcStatus::Error("ID Discord invalide (Erreur 10002)".into()));
                            } else {
                                let _ = status_tx.send(RpcStatus::WaitingDiscord);
                            }
                        }
                    }
                }
            }
        }
    });
}
