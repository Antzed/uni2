use clap::{Parser, Subcommand, Command, Arg};
use clap::CommandFactory;                  // lets us call Cli::command()
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Error as IoError,
    path::{Path, PathBuf},
    process::{Command as Cmd, exit},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

mod tui;

/* ---------- static CLI (built-ins) ---------- */

#[derive(Parser)]
#[command(name = "uni", version)]
struct Cli {
    /// Launch interactive TUI mode
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,
    
    #[command(subcommand)]
    command: Option<BuiltIn>,
}

#[derive(Subcommand)]
enum BuiltIn {
    Add    { path: PathBuf },
    Remove { name: String },
    List,
    Create { name: String },  
    Export { #[arg(default_value = "plugins.zip")] file: PathBuf },
    Import { file: PathBuf },
    EnsurePython {
        #[arg(long)]
        force: bool,
    },
}

/* ---------- manifest ---------- */

#[derive(Serialize, Deserialize)]
struct SubCmdMeta {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    name: String,
    description: String,
    version: String,
    #[serde(default)]
    commands: Vec<SubCmdMeta>,
}

/* ---------- plugin-directory helpers ---------- */

fn plugin_dir() -> PathBuf {
    ProjectDirs::from("", "", "mycli")
        .expect("cannot determine config dir")
        .config_dir()
        .join("plugins")
}

fn ensure_plugin_dir() -> Result<(), IoError> {
    fs::create_dir_all(plugin_dir())
}

/* ---------- add / remove / list ---------- */

fn validate_and_copy(path: &Path) -> Result<Manifest, Box<dyn std::error::Error>> {
    // ----- 1. run the candidate with --manifest ---------------------------
    let out = match path.extension().and_then(|e| e.to_str()) {
        Some("py") => Cmd::new("uv")
                        .args(["run", path.to_str().unwrap(), "--manifest"])
                        .output()?,
        _           => Cmd::new(path)
                        .arg("--manifest")
                        .output()?,
    };

    if !out.status.success() {
        return Err(format!(
            "plugin did not return valid manifest (exit {}):\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ).into());
    }

    let manifest: Manifest = serde_json::from_slice(&out.stdout)?;

    // ----- 2. copy the binary / script & chmod ----------------------------
    let dest_script = plugin_dir().join(&manifest.name);
    fs::copy(path, &dest_script)?;
    #[cfg(unix)]
    {
        let mut perm = fs::metadata(&dest_script)?.permissions();
        perm.set_mode(0o755);
        fs::set_permissions(&dest_script, perm)?;
    }

    // ----- 3. save manifest next to it ------------------------------------
    let dest_meta = plugin_dir().join(format!("{}.json", manifest.name));
    fs::write(dest_meta, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(manifest)
}

fn remove_plugin(name: &str) -> Result<(), IoError> {
    let dir = plugin_dir();
    let script = dir.join(name);
    let meta   = dir.join(format!("{}.json", name));
    if script.exists() { fs::remove_file(script)?; }
    if meta.exists()   { fs::remove_file(meta)?;   }
    Ok(())
}

pub fn list_plugins() -> Result<(), IoError> {
    for entry in fs::read_dir(plugin_dir())? {
        let p = entry?.path();
        if p.extension().and_then(|e| e.to_str()) == Some("json") {
            let data = fs::read(&p)?;
            let m: Manifest = serde_json::from_slice(&data)?;
            println!("- {}  (v{})  {}", m.name, m.version, m.description);
        }
    }
    Ok(())
}

/* ---------- dynamic CLI assembly ---------- */

fn load_manifests() -> Vec<Manifest> {
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(plugin_dir()) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(bytes) = fs::read(&p) {
                    if let Ok(m) = serde_json::from_slice::<Manifest>(&bytes) { out.push(m); }
                }
            }
        }
    }
    out
}

/* ---------- create CLI command template ---------- */


fn create_template(name: &str) -> std::io::Result<PathBuf> {
    let file_name = format!("{name}.py");
    let path      = std::env::current_dir()?.join(&file_name);

    // Simple one-shot write; will overwrite if the file exists
    const TEMPLATE: &str = r#"#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# dependencies = [
#     ///add you dependencies here
# ]
# ///
import sys, json, subprocess

MANIFEST = {
    "name": "<<NAME>>",
    "description": "Describe what this plugin does",
    "version": "0.1.0",
    "commands": [
        { "name": "run",    "description": "Run the job" },
        { "name": "status", "description": "Show status" }
    ]
}

def run_cmd(cmd: list[str]) -> None:
    """Run a shell command and stream its output; abort on failure."""
    result = subprocess.run(cmd, check=False, text=True)
    if result.returncode != 0:
        sys.exit(result.returncode);

def manifest():
    print(json.dumps(MANIFEST))
    sys.exit(0)

def run(args):
    print("Running <<NAME>> with", args)

def status(args):
    print("<<NAME>> status:", args)

def main():
    cmds = {"run": run, "status": status}
    sub  = sys.argv[1] if len(sys.argv) > 1 else None
    if sub in cmds:
        cmds[sub](sys.argv[2:])
    else:
        print("usage: {0} {{run|status}} …".format(MANIFEST["name"]))

if __name__ == "__main__":
    if "--manifest" in sys.argv:
        manifest()
    else:
        main()
"#;

    let contents = TEMPLATE.replace("<<NAME>>", name);
    std::fs::write(&path, contents)?;            /* std::fs::write does the create/truncate in one step :contentReference[oaicite:4]{index=4} */

    // Make it executable on Unix; ignored on Windows
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&path)?.permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&path, p)?;
    }

    Ok(path)
}

/* ---------- export CLI plugin commands ---------- */


fn export_plugins(zip_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use zip::write::FileOptions;
    use std::io::Write;

    let file = std::fs::File::create(zip_path)?;        // std::fs::File::create :contentReference[oaicite:2]{index=2}
    let mut zip = zip::ZipWriter::new(file);            // ZipWriter API :contentReference[oaicite:3]{index=3}
    let opts = FileOptions::default().unix_permissions(0o644);

    for entry in std::fs::read_dir(plugin_dir())? {     // read_dir iteration :contentReference[oaicite:4]{index=4}
        let p = entry?.path();
        if p.is_file() {
            let name = p.file_name().unwrap().to_string_lossy();
            zip.start_file(name, opts)?;                // each .py / .json becomes one entry
            let data = std::fs::read(&p)?;
            zip.write_all(&data)?;
        }
    }
    zip.finish()?;                                     // flush central directory
    println!("📦  Exported plugins to {}", zip_path.display());
    Ok(())
}

/* ---------- import CLI plugin commands ---------- */


fn import_plugins(zip_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::read::ZipArchive::new(file)?;               // :contentReference[oaicite:0]{index=0}

    // 1) unpack everything into an auto-cleaning temp dir
    let tmp = tempfile::tempdir()?;                                    // :contentReference[oaicite:1]{index=1}
    archive.extract(&tmp)?;                                            // single call does the loop for us :contentReference[oaicite:2]{index=2}

    // 2) walk the temp dir and feed every NON-JSON file to the validator
    for entry in std::fs::read_dir(&tmp)? {                            // :contentReference[oaicite:3]{index=3}
        let p = entry?.path();
        if p.extension().and_then(|e| e.to_str()) == Some("json") {    // skip manifests
            continue;
        }
        if !p.is_file() { continue; }                                  // guard against stray dirs

        match validate_and_copy(&p) {                                  // reuse your existing checks
            Ok(m) => println!("➕  Imported {}", m.name),
            Err(e) => eprintln!("⚠️  Skipped {}: {e}", p.display()),
        }
    }
    Ok(())
}

/* ---------- check if python is installed ---------- */

fn current_python_version() -> Option<String> {
    let candidates = ["python3", "python"];
    for exe in &candidates {
        if let Ok(out) = Cmd::new(exe).arg("--version").output() {
            // stdout on *nix, stderr on Windows; concatenate for safety
            let buf = [out.stdout, out.stderr].concat();
            let text = String::from_utf8_lossy(&buf);
            // expect `Python 3.13.3`
            if text.starts_with("Python") {
                // second word is version
                return text.split_whitespace().nth(1).map(|s| s.to_string());
            }
        }
    }
    None
}

/* ---------- check if uv is installed ---------- */
fn current_uv_version() -> Option<String> {
    if let Ok(out) = Cmd::new("uv").arg("--version").output() {
        // output is like `uv 0.7.14`
        let text = String::from_utf8_lossy(&out.stdout);
        if text.starts_with("uv ") {
            return text.split_whitespace().nth(1).map(str::to_owned);
        }
    }
    None
}

fn install_python() -> Result<(), Box<dyn std::error::Error>> {
    let target = "3.13.3";
    let os = std::env::consts::OS;

    match os {
        "windows" => {
            // prefer winget (Win 11 / Server 2022)
            if Cmd::new("where").arg("winget").output().is_ok() {
                let status = Cmd::new("winget")
                    .args(["install", "--id=Python.Python.3.13", "-e"])
                    .status()?;
                if status.success() { return Ok(()); }
            }
            // fall back to Chocolatey
            let status = Cmd::new("choco")
                .args(["install", "python313", "--yes"])
                .status()?;
            if status.success() { return Ok(()); }
            Err("winget/choco installation failed".into())
        }
        "macos" => {
            if Cmd::new("which").arg("brew").status()?.success() {
                let status = Cmd::new("brew")
                    .args(["install", "python@3.13"])
                    .status()?;
                if status.success() { return Ok(()); }
            }
            // fallback: pyenv
            install_with_pyenv(target)
        }
        _ /* linux, bsd, etc. */ => install_with_pyenv(target),
    }
}

fn install_with_pyenv(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    // install pyenv if missing
    if Cmd::new("which").arg("pyenv").status()?.success() == false {
        println!("→ installing pyenv (curl | bash) …");
        Cmd::new("bash")
            .arg("-c")
            .arg("curl -s https://pyenv.run | bash")
            .status()?;
        // user must add shims to PATH; best-effort reload
        let home = std::env::var("HOME")?;
        let old  = std::env::var("PATH").unwrap_or_default();
        // # Safety
        // `set_var` is unsafe because changing the environment is racy.  This CLI is
        // single-threaded after this point, so it is sound here.
        unsafe {
            std::env::set_var("PATH", format!("{home}/.pyenv/bin:{home}/.pyenv/shims:{old}"));
        }
    }
    println!("→ pyenv install {version}");
    let status = Cmd::new("pyenv").args(["install", "-s", version]).status()?;
    if !status.success() {
        return Err("pyenv failed to build Python".into());
    }
    // make it the global default so `python3` finds it
    Cmd::new("pyenv").args(["global", version]).status()?;
    Ok(())
}

fn install_uv() -> Result<(), Box<dyn std::error::Error>> {
    let os = std::env::consts::OS;

    match os {
        "windows" => {
            // PowerShell one-liner
            let script = r#"irm https://astral.sh/uv/install.ps1 | iex"#;
            let status = Cmd::new("powershell")
                .args(["-ExecutionPolicy", "ByPass", "-c", script])
                .status()?;
            if status.success() { return Ok(()); }
            Err("PowerShell uv install failed".into())
        }
        "macos" | "linux" => {
            // Prefer Homebrew if present (instant bottles) :contentReference[oaicite:3]{index=3}
            // if Cmd::new("which").arg("brew").status()?.success() &&
            //    Cmd::new("brew").args(["install", "uv"]).status()?.success() {
            //     return Ok(())
            // }
            // Fallback: official standalone installer (curl / wget) :contentReference[oaicite:4]{index=4}
            let curl_ok = Cmd::new("bash")
                .arg("-c")
                .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
                .status()?
                .success();
            if curl_ok { return Ok(()); }

            let wget_ok = Cmd::new("bash")
                .arg("-c")
                .arg("wget -qO- https://astral.sh/uv/install.sh | sh")
                .status()?
                .success();
            if wget_ok { return Ok(()); }

            Err("curl/wget uv install failed".into())
        }
        _ => Err("unsupported OS for uv install".into()),
    }
}




/* ---------- CLI command execution from TUI ---------- */

fn execute_cli_command(command_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command string (e.g., "uni git status" -> ["git", "status"])
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    
    if parts.is_empty() || parts[0] != "uni" {
        return Err("Invalid command format".into());
    }
    
    if parts.len() < 2 {
        // Just "uni" - show help
        build_cli().print_help()?;
        println!();
        return Ok(());
    }
    
    let command_name = parts[1];
    let remaining_args: Vec<&str> = parts[2..].to_vec();
    
    // Check if it's a built-in command
    match command_name {
        "add" => {
            if remaining_args.is_empty() {
                print!("Enter plugin file path: ");
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let path = PathBuf::from(input.trim());
                let m = validate_and_copy(&path)?;
                println!("Added plugin `{}` v{}", m.name, m.version);
            } else {
                let path = PathBuf::from(remaining_args[0]);
                let m = validate_and_copy(&path)?;
                println!("Added plugin `{}` v{}", m.name, m.version);
            }
        }
        "remove" => {
            if remaining_args.is_empty() {
                print!("Enter plugin name to remove: ");
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let name = input.trim();
                remove_plugin(name)?;
                println!("Removed plugin `{}`", name);
            } else {
                let name = remaining_args[0];
                remove_plugin(name)?;
                println!("Removed plugin `{}`", name);
            }
        }
        "list" => {
            list_plugins()?;
        }
        "create" => {
            if remaining_args.is_empty() {
                print!("Enter plugin name: ");
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let name = input.trim();
                match create_template(name) {
                    Ok(p) => {
                        println!(
                            "Created template at {}\n\
                            ->  vim {}   # edit, test, iterate\n\
                            ->  uni add {}   # register once ready",
                            p.display(), p.display(), p.display()
                        );
                    }
                    Err(e) => eprintln!("Failed to write template: {e}"),
                }
            } else {
                let name = remaining_args[0];
                match create_template(name) {
                    Ok(p) => {
                        println!(
                            "Created template at {}\n\
                            ->  vim {}   # edit, test, iterate\n\
                            ->  uni add {}   # register once ready",
                            p.display(), p.display(), p.display()
                        );
                    }
                    Err(e) => eprintln!("Failed to write template: {e}"),
                }
            }
        }
        "export" => {
            let file_path = if remaining_args.is_empty() {
                print!("Enter export file path (default: plugins.zip): ");
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();
                if input.is_empty() {
                    PathBuf::from("plugins.zip")
                } else {
                    PathBuf::from(input)
                }
            } else {
                PathBuf::from(remaining_args[0])
            };
            export_plugins(&file_path)?;
        }
        "import" => {
            if remaining_args.is_empty() {
                print!("Enter import file path: ");
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let path = PathBuf::from(input.trim());
                import_plugins(&path)?;
            } else {
                let path = PathBuf::from(remaining_args[0]);
                import_plugins(&path)?;
            }
        }
        "ensure-python" => {
            let force = remaining_args.contains(&"--force");
            
            /* ---------- 3.1 ensure CPython 3.13.3 ---------- */
            let need_python = match current_python_version() {
                Some(v) if v == "3.13.3" && !force => {
                    println!("✅ Python 3.13.3 already installed"); false
                }
                Some(v) => { println!("ℹ️  Found Python {v}, upgrading to 3.13.3"); true }
                None     => { println!("🚫 No python3 – installing 3.13.3"); true }
            };
            if need_python {
                match install_python() {
                    Ok(_)  => println!("🎉 Python 3.13.3 ready ✔"),
                    Err(e) => { eprintln!("❌ Python install failed: {e}"); return Ok(()); }
                }
            }

            /* ---------- 3.2 ensure uv ---------- */
            match current_uv_version() {
                Some(v) => println!("✅ uv {v} already installed"),
                None => {
                    println!("→ installing uv …");
                    match install_uv() {
                        Ok(_)  => println!("🎉 uv installed ✔"),
                        Err(e) => eprintln!("❌ uv install failed: {e}"),
                    }
                }
            }
        }
        _ => {
            // It's a plugin command - execute it with the remaining arguments
            execute_plugin_command(command_name, &remaining_args)?;
        }
    }
    
    Ok(())
}

fn execute_plugin_command(plugin_name: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let script = plugin_dir().join(plugin_name);
    
    if !script.exists() {
        return Err(format!("Plugin '{}' not found", plugin_name).into());
    }
    
    // If no arguments provided, prompt for them
    let final_args = if args.is_empty() {
        // Check if this plugin has subcommands
        let manifests = load_manifests();
        if let Some(manifest) = manifests.iter().find(|m| m.name == plugin_name) {
            if !manifest.commands.is_empty() {
                println!("Available subcommands for {}:", plugin_name);
                for cmd in &manifest.commands {
                    println!("  {} - {}", cmd.name, cmd.description);
                }
                print!("Enter subcommand and arguments: ");
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().split_whitespace().map(|s| s.to_string()).collect()
            } else {
                // No subcommands, prompt for arguments
                print!("Enter arguments for {} (or press Enter for none): ", plugin_name);
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().is_empty() {
                    Vec::new()
                } else {
                    input.trim().split_whitespace().map(|s| s.to_string()).collect()
                }
            }
        } else {
            // Plugin not found in manifests, prompt for arguments
            print!("Enter arguments for {} (or press Enter for none): ", plugin_name);
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().is_empty() {
                Vec::new()
            } else {
                input.trim().split_whitespace().map(|s| s.to_string()).collect()
            }
        }
    } else {
        args.iter().map(|s| s.to_string()).collect()
    };
    
    // Execute the plugin with the arguments
    let status = Cmd::new(&script).args(&final_args).status()?;
    exit(status.code().unwrap_or(1));
}

fn build_cli() -> Command {
    let mut cmd = Cli::command();  // static built-ins

    let trailing = Arg::new("args")
        .num_args(..)
        .trailing_var_arg(true)          // captures --flags etc. :contentReference[oaicite:1]{index=1}
        .allow_hyphen_values(true)
        .help("arguments forwarded to the plugin");

    for m in load_manifests() {          // parses *.json on disk
        // leak top-level strings
        let pname: &'static str = Box::leak(m.name.clone().into_boxed_str());
        let pdesc: &'static str = Box::leak(m.description.clone().into_boxed_str());

        let mut plug = Command::new(pname).about(pdesc);

        for sc in &m.commands {
            let sname: &'static str = Box::leak(sc.name.clone().into_boxed_str());
            let sdesc: &'static str = Box::leak(sc.description.clone().into_boxed_str());

            plug = plug.subcommand(
                Command::new(sname).about(sdesc).arg(trailing.clone())
            );                           // nested sub-commands :contentReference[oaicite:2]{index=2}
        }

        // If no commands declared, still add trailing args at top level.
        if m.commands.is_empty() {
            plug = plug.arg(trailing.clone());
        }

        cmd = cmd.subcommand(plug);      // insert into tree
    }
    cmd
}

/* ---------- main ---------- */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ensure_plugin_dir()?;

    // We need matches twice: once for built-ins, once for plugins
    let matches = build_cli().get_matches();

    // Check if interactive mode is requested
    if matches.get_flag("interactive") {
        match tui::run_tui()? {
            tui::TuiResult::Exit => return Ok(()),
            tui::TuiResult::ExecuteCommand(command) => {
                // Parse and execute the command from TUI
                return execute_cli_command(&command);
            }
        }
    }

    // 1) Handle built-in subcommands if any
    if let Some(("add",  sub_m)) = matches.subcommand() {
        let path = sub_m.get_one::<PathBuf>("path").unwrap();
        let m = validate_and_copy(path)?;
        println!("Added plugin `{}` v{}", m.name, m.version);
        return Ok(());
    }
    if let Some(("remove", sub_m)) = matches.subcommand() {
        let name = sub_m.get_one::<String>("name").unwrap();
        remove_plugin(name)?;
        println!("Removed plugin `{}`", name);
        return Ok(());
    }
    if let Some(("list", _)) = matches.subcommand() {
        list_plugins()?;
        return Ok(());
    }
    if let Some(("create", sub_m)) = matches.subcommand() {
        let name = sub_m.get_one::<String>("name").unwrap();
        match create_template(name) {
            Ok(p) => {
                println!(
                    "Created template at {}\n\
                    ->  vim {}   # edit, test, iterate\n\
                    ->  mycli add {}   # register once ready",
                    p.display(), p.display(), p.display()
                );
            }
            Err(e) => eprintln!("Failed to write template: {e}"),
        }
        return Ok(());
    }

    if let Some(("export", sub)) = matches.subcommand() {
        let path = sub.get_one::<PathBuf>("file").unwrap();
        export_plugins(path)?;
        return Ok(());
    }

    if let Some(("import", sub)) = matches.subcommand() {
        let path = sub.get_one::<PathBuf>("file").unwrap();
        import_plugins(path)?;
        return Ok(());
    }

    if let Some(("ensure-python", sub_m)) = matches.subcommand() {
        let force = *sub_m.get_one::<bool>("force").unwrap();

        /* ---------- 3.1 ensure CPython 3.13.3 ---------- */
        let need_python = match current_python_version() {
            Some(v) if v == "3.13.3" && !force => {
                println!("✅ Python 3.13.3 already installed"); false
            }
            Some(v) => { println!("ℹ️  Found Python {v}, upgrading to 3.13.3"); true }
            None     => { println!("🚫 No python3 – installing 3.13.3"); true }
        };
        if need_python {
            match install_python() {
                Ok(_)  => println!("🎉 Python 3.13.3 ready ✔"),
                Err(e) => { eprintln!("❌ Python install failed: {e}"); return Ok(()); }
            }
        }

        /* ---------- 3.2 ensure uv ---------- */
        match current_uv_version() {
            Some(v) => println!("✅ uv {v} already installed"),
            None => {
                println!("→ installing uv …");
                match install_uv() {
                    Ok(_)  => println!("🎉 uv installed ✔"),
                    Err(e) => eprintln!("❌ uv install failed: {e}"),
                }
            }
        }
        return Ok(());
    }


    // 2) Otherwise it must be a dynamically registered plugin
    if let Some((pname, pm)) = matches.subcommand() {
        // Ignore built-ins already handled above -----------------------------
        // Gather args and possible nested subcommand
        let mut argv: Vec<&std::ffi::OsStr> = Vec::new();

        if let Some((sname, sm)) = pm.subcommand() {
            argv.push(std::ffi::OsStr::new(sname));           // first token = subcmd
            argv.extend(sm.get_raw("args").unwrap_or_default());
        } else {
            argv.extend(pm.get_raw("args").unwrap_or_default());
        }

        let script = plugin_dir().join(pname);
        let status = Cmd::new(script).args(&argv).status()?;
        exit(status.code().unwrap_or(1));
    }

    // No subcommand at all → print help
    build_cli().print_help()?;
    println!();
    Ok(())
}
