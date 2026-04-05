use std::process::{Command, Stdio};
use std::time::Duration;
use tauri::{Emitter, Manager};
use tokio::time::sleep;

fn get_env_var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

const SPLASH_DATA_URL: &str = r#"data:text/html,
<html>
<body style="background:%23050505; color:%23eee; font-family:monospace; margin:0; display:flex; flex-direction:column; align-items:center; justify-content:center; height:100vh; border:1px solid %23222; overflow:hidden;">
    <div style="font-size:10px; color:%23444; margin-bottom:15px; letter-spacing:2px;">MATRIX_ORCHESTRATOR_V2</div>
    <div id="status" style="font-size:13px; text-transform:uppercase; color:%23d44; font-weight:bold;">SYSTEM_BOOT_INIT</div>
    <div style="width:150px; height:2px; background:%23111; margin-top:20px; position:relative; overflow:hidden;">
        <div style="width:60px; height:100%; background:%23d44; position:absolute; animation: progress 1.5s infinite ease-in-out;"></div>
    </div>
    <style>@keyframes progress { from { left: -60px; } to { left: 150px; } }</style>
    <script>
        const { listen } = window.__TAURI__.event;
        listen('status-update', (event) => {
            document.getElementById('status').textContent = event.payload;
        });
    </script>
</body>
</html>"#;

async fn container_exists(name: &str) -> bool {
    let output = Command::new("docker")
        .args(["ps", "-a", "-q", "-f", &format!("name=^/{}$", name)])
        .output();

    match output {
        Ok(out) => !out.stdout.is_empty(),
        Err(_) => false,
    }
}

async fn is_docker_active() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn handle_fatal_error(window: &tauri::WebviewWindow, code: &str, detail: &str, instruction: &str) {
    let html = format!(
        "data:text/html,<html><body style='background:%23050505;color:%23d44;font-family:monospace;margin:0;display:flex;align-items:center;justify-content:center;height:100vh;'>\
        <div style='padding:40px;border:1px solid %23d44;background:%23000;width:500px;'>\
            <h2 style='margin:0 0 10px 0; font-size:20px;'>[SYSTEM_OFFLINE: {code}]</h2>\
            <hr style='border:none; border-top:1px solid %23333; margin:20px 0;'>\
            <p style='color:%23ccc;font-size:14px;line-height:1.6;'>{detail}</p>\
            <div style='background:%23111;padding:15px;border-left:3px solid %23d44;color:%230f0;margin-top:20px;'><code>$ {instruction}</code></div>\
        </div></body></html>",
        code = code, detail = detail, instruction = instruction
    );
    let _ = window.navigate(reqwest::Url::parse(&html).unwrap());
    let _ = window.show();
}

#[tokio::main]
pub async fn run() {
    let container_name = get_env_var("CONTAINER_NAME", "f5-tts-rocm");
    let gradio_url = get_env_var("GRADIO_URL", "http://localhost:7860");

    tauri::Builder::default()
        .setup(move |app| {
            let main_window = app
                .get_webview_window("main")
                .expect("main window not found");
            let splash_window = app
                .get_webview_window("splash")
                .expect("splash window not found");
            let _ = splash_window.navigate(reqwest::Url::parse(SPLASH_DATA_URL).unwrap());

            let g_url = gradio_url.clone();
            let c_name = container_name.clone();

            tokio::spawn(async move {
                let _ = splash_window.emit("status-update", "VERIFYING_DOCKER_RUNTIME");
                if !is_docker_active().await {
                    let _ = splash_window.close();
                    handle_fatal_error(
                        &main_window,
                        "DOCKER_UNREACHABLE",
                        "Daemon down.",
                        "sudo systemctl start docker",
                    );
                    return;
                }

                let _ = splash_window.emit("status-update", "ORCHESTRATING_STACK");

                let success = if container_exists(&c_name).await {
                    Command::new("docker")
                        .args(["start", &c_name])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                } else {
                    Command::new("docker")
                        .args(["compose", "up", "-d"])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                };

                if !success {
                    let _ = splash_window.close();
                    handle_fatal_error(
                        &main_window,
                        "STACK_INIT_FAILURE",
                        "Failed to start container.",
                        "docker ps -a",
                    );
                    return;
                }

                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(2))
                    .build()
                    .unwrap();

                for i in 1..=120 {
                    let _ = splash_window.emit("status-update", format!("SYNCING_PORT_{}S", i));
                    if let Ok(res) = client.get(&g_url).send().await {
                        if res.status().is_success() {
                            let _ = main_window.navigate(reqwest::Url::parse(&g_url).unwrap());
                            sleep(Duration::from_millis(1000)).await;
                            let _ = splash_window.close();
                            let _ = main_window.show();
                            return;
                        }
                    }
                    sleep(Duration::from_secs(1)).await;
                }

                let _ = splash_window.close();
                handle_fatal_error(
                    &main_window,
                    "ENGINE_TIMEOUT",
                    "Sync failed.",
                    &format!("docker logs {}", c_name),
                );
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
