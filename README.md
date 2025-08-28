# Booschnie CLI

A command-line tool for uploading and downloading files to/from a VPS server, with specialized SQLite database deployment functionality.

## Features

- **Upload files**: Transfer multiple files from local machine to VPS
- **Download files**: Download files from VPS to local machine  
- **SQLite dump & deploy**: Create database dumps, upload to VPS, and deploy with automatic backup and rollback

## Prerequisites

- Rust (for building the tool)
- `sqlite3` command-line tool (for database operations)
- SSH access to your VPS server

### Installing sqlite3

**macOS:**
```bash
brew install sqlite3
```

**Ubuntu/Debian:**
```bash
sudo apt-get install sqlite3
```

**Windows:**
Download from [SQLite Download Page](https://www.sqlite.org/download.html)

## Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd booschnie-cli
```

2. Build the project:
```bash
cargo build --release
```

3. The executable will be available at `target/release/booschnie-cli`

## Configuration

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Edit `.env` with your VPS and database settings:

### Required Variables

```env
# VPS Connection
VPS_HOST=your-vps-ip-or-domain.com
VPS_USER=your-username

# Authentication (choose one method)
VPS_PASSWORD=your-password
# OR
VPS_KEY_PATH=/path/to/your/private/key
VPS_KEY_PASSPHRASE=optional-passphrase

# SQLite Database Settings (for option 3)
LOCAL_SQLITE_PATH=/path/to/your/local/database.db
REMOTE_SQLITE_DIR=/home/admin/app/data
REMOTE_SQLITE_NAME=production.db
```

### Authentication Methods

**Password Authentication:**
- Set `VPS_PASSWORD` with your server password

**Key-based Authentication (recommended):**
- Set `VPS_KEY_PATH` to your private key file
- Set `VPS_KEY_PASSPHRASE` if your key has a passphrase

## Usage

Run the CLI tool:
```bash
./target/release/booschnie-cli
```

### Menu Options

#### 1. Upload file(s)
- Enter local file paths (space-separated)
- Specify remote directory path
- Files are uploaded to the VPS

#### 2. Download file(s) 
- Enter remote file paths (space-separated)
- Specify local directory (default: current directory)
- Files are downloaded from VPS

#### 3. SQLite dump and deploy
Comprehensive database deployment with safety features:

**Process:**
1. **Create dump**: Generates SQL dump from local SQLite database
2. **Connect to VPS**: Establishes secure SFTP/SSH connection
3. **Backup existing**: Creates timestamped backup of current remote database
4. **Upload dump**: Transfers SQL dump file to VPS
5. **Deploy**: Executes dump to create new database
6. **Cleanup**: Removes temporary dump files
7. **Rollback**: Automatically restores backup if deployment fails

**Safety Features:**
- Automatic backup before deployment
- Rollback on failure
- Timestamped backups for recovery
- Error handling at each step

**Example workflow:**
```
Local DB: /Users/me/myapp/database.db
→ Creates: dump_20240101_120000.sql
→ Uploads to: /home/admin/app/data/
→ Backs up: production.db → backup_production_20240101_120000.db
→ Deploys: dump_20240101_120000.sql → production.db
```

## Environment Variables Reference

| Variable | Required | Description | Example |
|----------|----------|-------------|---------|
| `VPS_HOST` | ✅ | VPS server hostname/IP | `server.example.com` |
| `VPS_USER` | ✅ | SSH username | `admin` |
| `VPS_PASSWORD` | ⚠️* | SSH password | `mypassword` |
| `VPS_KEY_PATH` | ⚠️* | Private key file path | `/home/user/.ssh/id_rsa` |
| `VPS_KEY_PASSPHRASE` | ❌ | Key passphrase (if needed) | `mykeypass` |
| `LOCAL_SQLITE_PATH` | ✅** | Local database file path | `/app/data/app.db` |
| `REMOTE_SQLITE_DIR` | ✅** | Remote database directory | `/var/www/app/db` |
| `REMOTE_SQLITE_NAME` | ❌ | Remote database filename | `production.db` |

*Either `VPS_PASSWORD` or `VPS_KEY_PATH` is required
**Required only for SQLite functionality (option 3)

## Error Handling

The tool includes comprehensive error handling:

- **Connection failures**: Clear error messages for SSH/SFTP issues
- **File not found**: Validation of local and remote file paths  
- **Permission errors**: Helpful messages for access issues
- **Database errors**: SQLite command execution error reporting
- **Automatic rollback**: Failed deployments restore previous database

## Security Best Practices

1. **Use key-based authentication** instead of passwords when possible
2. **Restrict file permissions** on your `.env` file:
   ```bash
   chmod 600 .env
   ```
3. **Use dedicated deployment user** on VPS with limited privileges
4. **Keep backups**: The tool creates backups, but consider additional backup strategies

## Troubleshooting

### Common Issues

**"sqlite3 command not found"**
- Install SQLite3 tools on your system
- Ensure `sqlite3` is in your PATH

**"Permission denied" for SSH**
- Check your SSH credentials
- Verify key file permissions (should be 600)
- Confirm user has access to remote directories

**"Failed to create backup"**
- Ensure user has write permissions in remote directory
- Check available disk space on VPS

**Database deployment fails**
- Verify `sqlite3` is installed on VPS
- Check that remote directory exists
- Ensure database file isn't locked by another process

### Debug Mode

Set environment variable for verbose logging:
```bash
RUST_LOG=debug ./target/release/booschnie-cli
```

## Development

### Building from Source
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Adding Features
The codebase is structured with separate functions for each operation:
- `handle_upload_interactive()`
- `handle_download_interactive()`  
- `handle_sqlite_dump_deploy()`

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]