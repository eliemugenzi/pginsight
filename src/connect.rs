//! Interactive connection-credential prompt shown before the TUI starts.

use std::io::{self, Write};

use anyhow::{anyhow, Result};
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::cli::Cli;

pub struct Credentials {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
    pub dbname: String,
}

/// Build credentials from CLI args / env vars, prompting for anything missing.
pub fn prompt(cli: &Cli) -> Result<Credentials> {
    let sys_user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "postgres".into());

    let def_host = cli.host.as_deref().unwrap_or("localhost").to_string();
    let def_port = cli.port.unwrap_or(5432);
    let def_user = cli.username.as_deref().unwrap_or(&sys_user).to_string();

    // PGPASSWORD env var silently pre-fills the password (same as psql)
    let pg_password = std::env::var("PGPASSWORD").ok();

    let mut stdout = io::stdout();

    if cli.yes {
        // Non-interactive: use defaults / CLI args as-is, print a summary line.
        let dbname = cli
            .dbname
            .clone()
            .unwrap_or_else(|| def_user.clone());
        let password = if cli.no_password { None } else { pg_password };

        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::Cyan),
            Print("  pgmon"),
            ResetColor,
            SetForegroundColor(Color::DarkGrey),
            Print("  PostgreSQL Monitor\n"),
            ResetColor,
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "  Connecting to {}@{}:{}/{} …\n\n",
                def_user, def_host, def_port, dbname
            )),
            ResetColor,
        )?;

        return Ok(Credentials {
            host: def_host,
            port: def_port,
            user: def_user,
            password,
            dbname,
        });
    }

    // ── Interactive form ────────────────────────────────────────────────────
    execute!(
        stdout,
        Print("\n"),
        SetForegroundColor(Color::Cyan),
        Print("  pgmon"),
        ResetColor,
        Print("  "),
        SetForegroundColor(Color::DarkGrey),
        Print("PostgreSQL Monitor\n"),
        Print("  ─────────────────────────────────────────────────\n\n"),
        ResetColor,
    )?;

    let host = field(&mut stdout, "Host", &def_host)?;

    let port_str = field(&mut stdout, "Port", &def_port.to_string())?;
    let port: u16 = port_str
        .parse()
        .unwrap_or(def_port);

    let user = field(&mut stdout, "Username", &def_user)?;

    let def_dbname = cli.dbname.as_deref().unwrap_or(&user).to_string();
    let dbname = field(&mut stdout, "Database", &def_dbname)?;

    let password = if cli.no_password {
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print("  Password     [skipped — --no-password]\n"),
            ResetColor,
        )?;
        None
    } else if let Some(pw) = pg_password {
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print("  Password     [from PGPASSWORD]\n"),
            ResetColor,
        )?;
        if pw.is_empty() { None } else { Some(pw) }
    } else {
        let pw = password_field(&mut stdout, "Password")?;
        if pw.is_empty() { None } else { Some(pw) }
    };

    execute!(
        stdout,
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(format!(
            "  Connecting to {}@{}:{}/{} …\n\n",
            user, host, port, dbname
        )),
        ResetColor,
    )?;

    Ok(Credentials { host, port, user, password, dbname })
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn field(stdout: &mut io::Stdout, label: &str, default: &str) -> Result<String> {
    execute!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("  {:<13}", label)),
        ResetColor,
    )?;

    if !default.is_empty() {
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("[{}] ", default)),
            ResetColor,
        )?;
    }

    stdout.flush()?;

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => return Err(anyhow!("EOF on stdin")),
        Err(e) if e.kind() == io::ErrorKind::Interrupted => {
            return Err(anyhow!("Interrupted"));
        }
        Err(e) => return Err(e.into()),
        Ok(_) => {}
    }

    let trimmed = input.trim().to_string();
    Ok(if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed
    })
}

fn password_field(stdout: &mut io::Stdout, label: &str) -> Result<String> {
    execute!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("  {:<13}", label)),
        ResetColor,
    )?;
    stdout.flush()?;

    enable_raw_mode()?;

    let mut password = String::new();

    let result = (|| -> Result<String> {
        loop {
            match read()? {
                Event::Key(key) => match (key.code, key.modifiers) {
                    (KeyCode::Enter, _) => break,
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Err(anyhow!("Interrupted"));
                    }
                    (KeyCode::Char(c), _) => {
                        password.push(c);
                        execute!(stdout, Print("*"))?;
                        stdout.flush()?;
                    }
                    (KeyCode::Backspace, _) => {
                        if !password.is_empty() {
                            password.pop();
                            execute!(
                                stdout,
                                cursor::MoveLeft(1),
                                Print(" "),
                                cursor::MoveLeft(1)
                            )?;
                            stdout.flush()?;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(password)
    })();

    disable_raw_mode()?;
    execute!(stdout, Print("\n"))?;
    stdout.flush()?;

    result
}
