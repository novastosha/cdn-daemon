use std::{ffi::OsString, time::Duration};
use windows_service::{
    define_windows_service,
    service::{
        ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::ServiceControlHandlerResult,
    service_dispatcher,
};

use watcher_logic::run_git_watcher;

const SERVICE_NAME: &str = "CDN Watcher Service";

define_windows_service!(ffi_service_main, my_service_main);

fn main() -> windows_service::Result<()> {
    if std::env::args().any(|arg| arg == "--run") {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    } else {
        let cmd = std::env::args().nth(1).unwrap_or_default();
        match cmd.as_str() {
            "install" => install_service(),
            "uninstall" => uninstall_service(),
            _ => {
                println!("Usage:");
                println!("  <exe> install");
                println!("  <exe> uninstall");
                println!("  <exe> --run");
                Ok(())
            }
        }
    }
}

fn my_service_main(_args: Vec<OsString>) {
    if let Err(e) = run_as_service() {
        eprintln!("Service failed: {:?}", e);
    }
}

fn run_as_service() -> windows_service::Result<()> {
    let status_handle = windows_service::service_control_handler::register(SERVICE_NAME, |_| {
        ServiceControlHandlerResult::NoError
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(10),
        process_id: None,
    })?;

    let repo = std::env::var("CDN_REPO_PATH").expect("CDN_REPO_PATH environment variable not set");
    Ok(run_git_watcher(repo))
}

fn install_service() -> windows_service::Result<()> {
    use std::{
        ffi::OsString,
        fs,
        path::Path,
    };
    use windows_service::service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType,
    };
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let system32_path = Path::new("C:\\Windows\\System32"); // Scream at me, I don't care.
    let dest_path = system32_path.join("cdn-watcher.exe");

    let current_exe = std::env::current_exe().expect("Failed to get current executable path");

    println!("Copying binary to {}", dest_path.display());
    fs::copy(&current_exe, &dest_path).expect("Failed to copy executable to system32");

    // Create the service manager
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;

    // Build service info
    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from("Watches CDN repository for file changes and runs git commands"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: dest_path.clone(),
        launch_arguments: vec![OsString::from("--run")],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    manager.create_service(&service_info, ServiceAccess::START)?;

    println!("Service installed successfully.");
    Ok(())
}

fn uninstall_service() -> windows_service::Result<()> {
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    use windows_service::service::ServiceAccess;

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::all())?;
    service.delete()?;
    println!("Service uninstalled.");
    Ok(())
}
