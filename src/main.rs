use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
//use dotenvy::dotenv;
use dotenvy::from_path;
use ssh2::Session;

fn main() -> io::Result<()> {
    // Load .env if present
    //dotenv().ok();
    let home = env::var("HOME").expect("HOME environment variable not set");
    let env_path = PathBuf::from(format!("{}/.booschnie_vps.env", home));
    from_path(&env_path).ok();

    let host = env::var("VPS_HOST").expect("VPS_HOST must be set in .env");
    let user = env::var("VPS_USER").expect("VPS_USER must be set in .env");
    let password = env::var("VPS_PASSWORD").ok();
    let key_path = env::var("VPS_KEY_PATH").ok();
    let key_passphrase = env::var("VPS_KEY_PASSPHRASE").ok();

    if password.is_none() && key_path.is_none() {
        panic!("Either VPS_PASSWORD or VPS_KEY_PATH must be set in .env");
    }

    loop {
        println!("\nVPS File Transfer Tool");
        println!("1. Upload file(s)");
        println!("2. Download file(s)");
        println!("3. SQLite dump and deploy");
        println!("4. Exit");
        print!("Select an option (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice: u32 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input. Please enter a number.");
                continue;
            }
        };

        match choice {
            1 => handle_upload_interactive(
                &host,
                &user,
                password.as_deref(),
                key_path.as_deref(),
                key_passphrase.as_deref(),
            )?,
            2 => handle_download_interactive(
                &host,
                &user,
                password.as_deref(),
                key_path.as_deref(),
                key_passphrase.as_deref(),
            )?,
            3 => handle_sqlite_dump_deploy(
                &host,
                &user,
                password.as_deref(),
                key_path.as_deref(),
                key_passphrase.as_deref(),
            )?,
            4 => break,
            _ => println!("Invalid option. Please choose 1, 2, 3, or 4."),
        }
    }

    Ok(())
}

fn establish_session(
    host: &str,
    user: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    key_passphrase: Option<&str>,
) -> Result<ssh2::Sftp, io::Error> {
    let tcp = std::net::TcpStream::connect(format!("{}:22", host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    if let Some(pw) = password {
        sess.userauth_password(user, pw)?;
    } else if let Some(kp) = key_path {
        let key_file = Path::new(kp);
        if key_file.exists() {
            sess.userauth_pubkey_file(user, None, key_file, key_passphrase)?;
        } else {
            panic!("Key file not found: {}", kp);
        }
    }

    Ok(sess.sftp()?)
}

fn handle_upload_interactive(
    host: &str,
    user: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    key_passphrase: Option<&str>,
) -> io::Result<()> {
    print!("Enter local file paths (space-separated): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let files: Vec<String> = input.trim().split_whitespace().map(String::from).collect();

    if files.is_empty() {
        println!("No files specified.");
        return Ok(());
    }

    print!("Enter remote directory path (e.g., /home/admin/uploads): ");
    io::stdout().flush()?;
    let mut remote_dir = String::new();
    io::stdin().read_line(&mut remote_dir)?;
    let remote_dir = remote_dir.trim().to_string();

    if remote_dir.is_empty() {
        println!("No remote directory specified.");
        return Ok(());
    }

    let sftp = establish_session(host, user, password, key_path, key_passphrase)?;

    for local_path_str in files {
        let local_path = Path::new(&local_path_str);
        if !local_path.is_file() {
            println!("Warning: Skipping {} (not a file)", local_path_str);
            continue;
        }

        let remote_path = format!(
            "{}/{}",
            remote_dir.trim_end_matches('/'),
            local_path.file_name().unwrap().to_str().unwrap()
        );
        let mut remote_file = sftp.create(Path::new(&remote_path))?;
        let mut local_file = File::open(local_path)?;

        io::copy(&mut local_file, &mut remote_file)?;
        println!("Uploaded {} to {}", local_path_str, remote_path);
    }

    Ok(())
}

fn handle_download_interactive(
    host: &str,
    user: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    key_passphrase: Option<&str>,
) -> io::Result<()> {
    print!("Enter remote file paths (space-separated): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let remote_files: Vec<String> = input.trim().split_whitespace().map(String::from).collect();

    if remote_files.is_empty() {
        println!("No files specified.");
        return Ok(());
    }

    print!("Enter local directory path (default: .): ");
    io::stdout().flush()?;
    let mut local_dir_str = String::new();
    io::stdin().read_line(&mut local_dir_str)?;
    let local_dir_str = local_dir_str.trim();
    let local_dir: PathBuf = if local_dir_str.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(local_dir_str)
    };

    let sftp = establish_session(host, user, password, key_path, key_passphrase)?;

    for remote_path_str in remote_files {
        let remote_path = Path::new(&remote_path_str);
        let local_path = local_dir.join(remote_path.file_name().unwrap());

        let mut remote_file = sftp.open(remote_path)?;
        let mut local_file = File::create(&local_path)?;

        io::copy(&mut remote_file, &mut local_file)?;
        println!("Downloaded {} to {}", remote_path_str, local_path.display());
    }

    Ok(())
}

fn handle_sqlite_dump_deploy(
    host: &str,
    user: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    key_passphrase: Option<&str>,
) -> io::Result<()> {
    // Get environment variables for SQLite paths
    let local_db_path = env::var("LOCAL_SQLITE_PATH")
        .expect("LOCAL_SQLITE_PATH must be set in .env (path to local SQLite database)");
    let remote_db_dir = env::var("REMOTE_SQLITE_DIR")
        .expect("REMOTE_SQLITE_DIR must be set in .env (directory path on VPS where database should be deployed)");
    let remote_db_name =
        env::var("REMOTE_SQLITE_NAME").unwrap_or_else(|_| "database.db".to_string());

    println!("Starting SQLite dump and deploy process...");
    println!("Local database: {}", local_db_path);
    println!("Remote directory: {}", remote_db_dir);
    println!("Remote database name: {}", remote_db_name);

    // Step 1: Create SQLite dump file
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let dump_filename = format!("dump_{}.sql", timestamp);
    let dump_path = PathBuf::from(&dump_filename);

    println!("\n1. Creating SQLite dump...");
    let dump_output = Command::new("sqlite3")
        .arg(&local_db_path)
        .arg(".dump")
        .output();

    match dump_output {
        Ok(output) => {
            if output.status.success() {
                std::fs::write(&dump_path, output.stdout)?;
                println!("✓ Dump created: {}", dump_filename);
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("✗ Failed to create dump: {}", error);
                return Ok(());
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to execute sqlite3 command: {}", e);
            eprintln!("Make sure sqlite3 is installed and in your PATH");
            return Ok(());
        }
    }

    // Step 2: Establish SFTP connection
    println!("\n2. Connecting to VPS...");
    let sftp = establish_session(host, user, password, key_path, key_passphrase)?;
    println!("✓ Connected to VPS");

    // Step 3: Create backup of existing database (if it exists)
    let remote_db_path = format!("{}/{}", remote_db_dir.trim_end_matches('/'), remote_db_name);
    let backup_db_path = format!(
        "{}/backup_{}_{}",
        remote_db_dir.trim_end_matches('/'),
        remote_db_name.replace(".db", ""),
        timestamp
    );

    println!("\n3. Creating backup of existing database...");
    match sftp.stat(Path::new(&remote_db_path)) {
        Ok(_) => {
            // Database exists, create backup
            match sftp.rename(
                Path::new(&remote_db_path),
                Path::new(&format!("{}.db", backup_db_path)),
                None,
            ) {
                Ok(_) => println!("✓ Backup created: {}.db", backup_db_path),
                Err(e) => {
                    eprintln!("✗ Failed to create backup: {}", e);
                    // Clean up dump file
                    std::fs::remove_file(&dump_path).ok();
                    return Ok(());
                }
            }
        }
        Err(_) => {
            println!("ℹ No existing database found, skipping backup");
        }
    }

    // Step 4: Upload dump file to VPS
    println!("\n4. Uploading dump file...");
    let remote_dump_path = format!("{}/{}", remote_db_dir.trim_end_matches('/'), dump_filename);
    let mut remote_dump_file = match sftp.create(Path::new(&remote_dump_path)) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("✗ Failed to create remote dump file: {}", e);
            // Restore backup if it exists
            if sftp
                .stat(Path::new(&format!("{}.db", backup_db_path)))
                .is_ok()
            {
                sftp.rename(
                    Path::new(&format!("{}.db", backup_db_path)),
                    Path::new(&remote_db_path),
                    None,
                )
                .ok();
                println!("Restored backup database");
            }
            std::fs::remove_file(&dump_path).ok();
            return Ok(());
        }
    };

    let mut local_dump_file = File::open(&dump_path)?;
    io::copy(&mut local_dump_file, &mut remote_dump_file)?;
    println!("✓ Dump file uploaded");

    // Step 5: Execute dump file on remote database
    println!("\n5. Applying dump to remote database...");

    // Create the new database by executing the dump
    let create_db_command = format!(
        "cd {} && sqlite3 {} < {}",
        remote_db_dir.trim_end_matches('/'),
        remote_db_name,
        dump_filename
    );

    // Execute command via SSH
    let tcp = std::net::TcpStream::connect(format!("{}:22", host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    if let Some(pw) = password {
        sess.userauth_password(user, pw)?;
    } else if let Some(kp) = key_path {
        let key_file = Path::new(kp);
        if key_file.exists() {
            sess.userauth_pubkey_file(user, None, key_file, key_passphrase)?;
        }
    }

    let mut channel = sess.channel_session()?;
    channel.exec(&create_db_command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_close()?;
    let exit_code = channel.exit_status()?;

    if exit_code == 0 {
        println!("✓ Database successfully updated from dump");

        // Clean up: remove dump file from remote
        sftp.unlink(Path::new(&remote_dump_path)).ok();
        println!("✓ Cleaned up remote dump file");

        // Remove backup after successful deployment (optional - comment out to keep backups)
        // sftp.unlink(Path::new(&format!("{}.db", backup_db_path))).ok();

        println!("\n🎉 SQLite dump and deploy completed successfully!");
        println!("Backup location: {}.db", backup_db_path);
    } else {
        eprintln!("✗ Failed to apply dump (exit code: {})", exit_code);
        eprintln!("Command output: {}", output);

        // Restore backup if deployment failed
        if sftp
            .stat(Path::new(&format!("{}.db", backup_db_path)))
            .is_ok()
        {
            sftp.rename(
                Path::new(&format!("{}.db", backup_db_path)),
                Path::new(&remote_db_path),
                None,
            )
            .ok();
            println!("✓ Restored backup database due to deployment failure");
        }

        // Clean up dump file
        sftp.unlink(Path::new(&remote_dump_path)).ok();
    }

    // Clean up local dump file
    std::fs::remove_file(&dump_path).ok();

    Ok(())
}
