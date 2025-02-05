use std::process::exit;
use std::env;
use std::process;
use actix_web::{web, App, HttpServer, Responder};
use actix_web::{HttpResponse, http::header};
use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MainConfig {
    #[serde(rename = "enableWebApp")]
    enable_web_app: bool,
    #[serde(rename = "webAppPort")]
    web_app_port: u16,
    #[serde(rename = "vncPort")]
    vnc_port: u16,
}

#[derive(Debug, Deserialize)]
struct WebAppConfig {
    #[serde(rename = "serviceName")]
    service_name: String,
    #[serde(rename = "serviceVersion")]
    service_version: String,
    #[serde(rename = "serviceDevelopmentState")]
    service_development_state: String,
    #[serde(rename = "serviceDescription")]
    service_description: String,
    #[serde(rename = "serviceAuthor")]
    service_author: String,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    #[serde(rename = "authEnabled")]
    auth_enabled: bool,
    #[serde(rename = "dbType")]
    db_type: String,
    #[serde(rename = "dbPath")]
    db_path: String,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct VMConfig {
    #[serde(rename = "qemu-args")]
    qemu_args: String,
    #[serde(rename = "qemu-ram")]
    qemu_ram: String,
    #[serde(rename = "qemu-cpu")]
    qemu_cpu: String,
    #[serde(rename = "qemu-command")]
    qemu_command: String,
    #[serde(rename = "network-adapter")]
    network_adapter: String,
    #[serde(rename = "machine-type")]
    machine_type: String,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    main: MainConfig,
    webapp: WebAppConfig,
    auth: AuthConfig,
    mysql: DatabaseConfig,
    mariadb: DatabaseConfig,
    mongodb: DatabaseConfig,
    postgresql: DatabaseConfig,
    vm: VMConfig,
}

async fn greet() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(r#"
            <h1>It looks like your UltiVM instance is running correctly!</h1>
            <p>If you see this message, your UltiVM instance is up and running. You can now connect to the VNC server at <b>localhost:5901</b> using a VNC client.</p>
            <p>If you want a web client for UltiVM, make sure that enableWebApp is set to true in the configuration file <i>(config.ini)</i>.</p>
        "#)
}

#[actix_web::main]
async fn start_webserver(config: AppConfig) -> std::io::Result<()> {
    let web_app_port = config.main.web_app_port;
    let vnc_port = config.main.vnc_port;
    let qemu_args = config.vm.qemu_args;
    let qemu_command = config.vm.qemu_command;
    let machine_type = config.vm.machine_type;

    println!("[SERVER] Web server starting at port {}...", web_app_port);
    std::thread::spawn(move || {
        let qemu_command = format!("{} -vnc :{} -machine {} {}", qemu_command, vnc_port - 5900, machine_type, qemu_args);
        println!("[QEMU] Starting virtual machine with VNC on port {}...", vnc_port);
        let output = process::Command::new("sh")
            .arg("-c")
            .arg(qemu_command)
            .output()
            .expect("Failed to execute QEMU command");
        println!("[QEMU] Trying to show error dialog using 'zenity'...");
        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            println!("[QEMU] Error: {}", error_message);
            eprintln!("[QEMU] Error: {}", error_message);
            let _zenity_output = process::Command::new("sh")
            .arg("-c")
            .arg(format!("zenity --error --title=\"Whoopsie...\" --text=\"We've encountered into an error...\n{}\nFor additional information, view what happened in the logs.\" --ok-label=\"Exit from UltiVM\"", error_message))
            .output()
            .expect("FAIL");
            exit(1);
        }
    });

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
    })
    .bind(format!("127.0.0.1:{}", web_app_port))?
    .run()
    .await
}

fn main() {
    let mut settings = Config::default();
    settings.merge(File::new("req/config.ini", FileFormat::Ini)).unwrap();
    let config: AppConfig = match settings.try_into() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            exit(1);
        }
    };

    println!("============================================");
    println!("=   WELCOME TO ULTIVM - A ONLINE VIRTUAL   =");
    println!("=         COLLABORATIVE MACHINE            =");
    println!("============================================");
    println!();
    if !std::path::Path::new("req/config.ini").exists() {
        eprintln!("E: Configuration file not found! You should go ahead and download the sample config.ini file from our repository (https://github.com/imguest24897-alt/UltiVM).");
        exit(1);
    }
    println!("Parsing QEMU (quick emulator) arguments: {}...", config.vm.qemu_args);

    let allowed_args = vec![
        "-hda", "-cdrom", "-drive", "-accel", "--enable-kvm", "-usb", "-device",
    ];
    let mut valid = true;
    let mut iter = config.vm.qemu_args.split_whitespace().peekable();

    while let Some(arg) = iter.next() {
        if allowed_args.contains(&arg) {
            match arg {
                "-hda" | "-cdrom" | "-drive" | "-accel" | "-device" => {
                    if iter.peek().is_none() {
                        valid = false;
                        break;
                    }
                    iter.next();
                }
                "--enable-kvm" | "-usb" => {
                    // don't need to check anything in this
                }
                _ => {
                    valid = false;
                    break;
                }
            }
        } else {
            valid = false;
            break;
        }
    }

    if valid {
        println!("Parsing success!");
        if config.main.enable_web_app {
            println!("W: Web application is not implemented, the page will be currently a status check page.");
            let _ = start_webserver(config);
        } else {
            std::thread::spawn(move || {
                let qemu_command = format!("{} -vnc :{} -machine {} {}", config.vm.qemu_command, config.main.vnc_port - 5900, config.vm.machine_type, config.vm.qemu_args);
                println!("[QEMU] Starting virtual machine with VNC on port {}...", config.main.vnc_port);
                let output = process::Command::new("sh")
                    .arg("-c")
                    .arg(qemu_command)
                    .output()
                    .expect("Failed to execute QEMU command");
                println!("[QEMU] Trying to show error dialog using 'zenity'...");
                if !output.status.success() {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    println!("[QEMU] Error: {}", error_message);
                    eprintln!("[QEMU] Error: {}", error_message);
                    let _zenity_output = process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("zenity --error --title=\"Whoopsie...\" --text=\"We've encountered into an error...\n{}\nFor additional information, view what happened in the logs.\" --ok-label=\"Exit from UltiVM\"", error_message))
                    .output()
                    .expect("FAIL");
                    exit(1);
                }
            }).join().unwrap();
        }
    } else {
        println!("E: Parsing failed! Please check the QEMU (quick emulator) arguments.");
    }
}
