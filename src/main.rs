use std::process::exit;
use std::env;
use std::process;
use actix_web::{web, App, HttpServer, Responder};
use actix_web::{HttpResponse, http::header};
use config::{Config, File, FileFormat};
use serde::Deserialize;
use reqwest::Error;

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
    #[serde(rename = "cpu-model")]
    cpu_model: String,
    #[serde(rename = "qemu-kvm-enabled")]
    qemu_kvm_enabled: bool,
    vga: String,
    #[serde(rename = "show-window")]
    show_window: bool,
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

const VERSION: &str = "0.0.1";

async fn greet() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(r#"
            <h1>It looks like your UltiVM instance is running correctly!</h1>
            <p>If you see this message, your UltiVM instance is up and running. You can now connect to the VNC server at <b>localhost:5901</b> using a VNC client.</p>
            <p>If you want a web client for UltiVM, make sure that enableWebApp is set to true in the configuration file <i>(config.ini)</i>.</p>
        "#)
}

async fn auth() -> impl Responder {
    HttpResponse::NotImplemented()
        .content_type("text/plain; charset=utf-8")
        .body("Auth is not implemented, sorry :(")
}

#[actix_web::main]
async fn start_webserver(config: AppConfig) -> std::io::Result<()> {
    let web_app_port = config.main.web_app_port;
    let vnc_port = config.main.vnc_port;
    let qemu_args = config.vm.qemu_args;
    let qemu_command = config.vm.qemu_command;
    let machine_type = config.vm.machine_type;
    let cpu_model = config.vm.cpu_model;
    let qemu_kvm_enabled = config.vm.qemu_kvm_enabled;
    let vga = config.vm.vga;
    let show_window = config.vm.show_window;
    let qemu_ram = config.vm.qemu_ram;
    let qemu_cpu = config.vm.qemu_cpu;
    let network_adapter = config.vm.network_adapter;

    println!("[SERVER] Web server starting at port {}...", web_app_port);
    std::thread::spawn(move || {
        let kvm_option = if qemu_kvm_enabled { "-enable-kvm" } else { "" };
        let display_option = if show_window { "-display gtk" } else { "" };
        let qemu_command = format!("{} -vnc :{} -machine {} -cpu {} -m {} -smp {} -net nic,model={} -vga {} {} {} {}", qemu_command, vnc_port - 5900, machine_type, cpu_model, qemu_ram, qemu_cpu, network_adapter, vga, kvm_option, display_option, qemu_args);
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
            .route("/auth", web::get().to(auth))
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
                "-hda" | "-cdrom" | "-drive" | "-accel" | "-device" | "-hdb" | "-hdc" | "-hdd" | "-usbdevice" | "-no-shutdown" | "-no-reboot" | "-nodefaults" | "-qmp" | "-qmp-pretty" | "-chardev" | "-gdb" | "netdev" | "-serial" | "-parallel" | "-soundhw" | "-audio" | "-append" | "-name" | "-k" => {
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
                let kvm_option = if config.vm.qemu_kvm_enabled { "-enable-kvm" } else { "" };
                let display_option = if config.vm.show_window { "-display gtk" } else { "" };
                let qemu_command = format!("{} -vnc :{} -machine {} -cpu {} -m {} -smp {} -net nic,model={} -vga {} {} {} {}", config.vm.qemu_command, config.main.vnc_port - 5900, config.vm.machine_type, config.vm.cpu_model, config.vm.qemu_ram, config.vm.qemu_cpu, config.vm.network_adapter, config.vm.vga, kvm_option, display_option, config.vm.qemu_args);
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
async fn check_for_updates() -> Result<(), Error> {
    let url = "https://api.github.com/repos/imguest24897-alt/UltiVM/releases/latest";
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "request")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let latest_version = response["tag_name"].as_str().unwrap_or("unknown");
    let update_title = response["name"].as_str().unwrap_or("No title");

    println!("Latest version: {}", latest_version);
    println!("Update title: {}", update_title);

    if latest_version != VERSION {
        println!("A new version of UltiVM is available: {} - {}", latest_version, update_title);
        println!("Please update to the latest version.");
    } else {
        println!("You are running the latest version of UltiVM.");
    }

    Ok(())
}

#[actix_web::main]
async fn actix_main() {
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
    if let Err(e) = check_for_updates().await {
        eprintln!("Failed to check for updates: {}", e);
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
                "-hda" | "-cdrom" | "-drive" | "-accel" | "-device" | "-hdb" | "-hdc" | "-hdd" | "-usbdevice" | "-no-shutdown" | "-no-reboot" | "-nodefaults" | "-qmp" | "-qmp-pretty" | "-chardev" | "-gdb" | "netdev" | "-serial" | "-parallel" | "-soundhw" | "-audio" | "-append" | "-name" | "-k" => {
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
                let kvm_option = if config.vm.qemu_kvm_enabled { "-enable-kvm" } else { "" };
                let display_option = if config.vm.show_window { "-display gtk" } else { "" };
                let qemu_command = format!("{} -vnc :{} -machine {} -cpu {} -m {} -smp {} -net nic,model={} -vga {} {} {} {}", config.vm.qemu_command, config.main.vnc_port - 5900, config.vm.machine_type, config.vm.cpu_model, config.vm.qemu_ram, config.vm.qemu_cpu, config.vm.network_adapter, config.vm.vga, kvm_option, display_option, config.vm.qemu_args);
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

    if let Err(e) = check_for_updates().await {
        eprintln!("Failed to check for updates: {}", e);
    }
}