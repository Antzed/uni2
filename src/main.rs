// use clap::{Parser, Subcommand};
// use clap::CommandFactory;          // For `Cli::command()` (help, completions, etc.)
// use directories::ProjectDirs;
// use serde::Deserialize;
// use std::{
//     fs,
//     io::Error as IoError,
//     os::unix::fs::PermissionsExt,
//     path::{Path, PathBuf},
//     process::{Command, exit},
// };

// /// Top-level parser; unknown words are forwarded to a plugin.
// #[derive(Parser)]
// #[command(name = "mycli", version, allow_external_subcommands = true)]
// struct Cli {
//     #[command(subcommand)]
//     command: Option<Commands>,
// }

// #[derive(Subcommand)]
// enum Commands {
//     /// Add a new script as a subcommand
//     Add { path: PathBuf },
//     /// Remove an existing plugin
//     Remove { name: String },
//     /// List installed plugins
//     List,
//     /// Everything else → external subcommand (`mycli foo …`)
//     #[command(external_subcommand)]
//     External(Vec<String>),          // first item = plugin name, rest = args
// }

// #[derive(Deserialize)]
// struct Manifest {
//     name: String,
//     description: String,
//     version: String,
// }

// /* ---------- helper utilities ---------- */

// fn plugin_dir() -> PathBuf {
//     ProjectDirs::from("", "", "mycli")
//         .expect("cannot determine config dir")
//         .config_dir()
//         .join("plugins")
// }

// fn ensure_plugin_dir() -> Result<(), IoError> {
//     fs::create_dir_all(plugin_dir())
// }

// fn validate_and_copy(path: &Path) -> Result<Manifest, Box<dyn std::error::Error>> {
//     let out = Command::new("python3").arg(path).arg("--manifest").output()?;    
//     if !out.status.success() {
//         return Err("plugin did not return valid manifest".into());
//     }

//     let manifest: Manifest = serde_json::from_slice(&out.stdout)?;
//     let dest = plugin_dir().join(&manifest.name);
//     fs::copy(path, &dest)?;
//     let mut perm = fs::metadata(&dest)?.permissions();
//     perm.set_mode(0o755);
//     fs::set_permissions(&dest, perm)?;
//     Ok(manifest)
// }

// fn list_plugins() -> Result<(), IoError> {
//     for entry in fs::read_dir(plugin_dir())? {
//         let path = entry?.path();
//         if path.is_file() {
//             println!("- {}", path.file_name().unwrap().to_string_lossy());
//         }
//     }
//     Ok(())
// }

// fn remove_plugin(name: &str) -> Result<(), IoError> {
//     let p = plugin_dir().join(name);
//     if p.exists() {
//         fs::remove_file(p)?;
//         println!("Removed plugin `{}`", name);
//     } else {
//         eprintln!("No such plugin: {}", name);
//     }
//     Ok(())
// }

// /* ---------- main ---------- */

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     ensure_plugin_dir()?;
//     let cli = Cli::parse();

//     match cli.command {
//         Some(Commands::Add { path }) => {
//             let m = validate_and_copy(&path)?;
//             println!("Added plugin `{}` v{}", m.name, m.version);
//         }
//         Some(Commands::List) => list_plugins()?,
//         Some(Commands::Remove { name }) => remove_plugin(&name)?,
//         Some(Commands::External(mut raw)) => {
//             // `raw[0]` = plugin name, remainder = args
//             if raw.is_empty() {
//                 Cli::command().print_help()?;
//                 exit(1);
//             }
//             let name = raw.remove(0);
//             let plugin_path = plugin_dir().join(&name);
//             if !plugin_path.exists() {
//                 eprintln!("Unknown command or plugin: `{}`", name);
//                 exit(1);
//             }
//             let status = Command::new(plugin_path).args(&raw).status()?;
//             exit(status.code().unwrap_or(1));
//         }
//         None => {
//             // No subcommand at all → show help
//             Cli::command().print_help()?;
//         }
//     }

//     Ok(())
// }

use clap::{Parser, Subcommand, Command, Arg};
use clap::CommandFactory;                  // lets us call Cli::command()
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Error as IoError,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command as Cmd, exit},
};

/* ---------- static CLI (built-ins) ---------- */

#[derive(Parser)]
#[command(name = "mycli", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<BuiltIn>,
}

#[derive(Subcommand)]
enum BuiltIn {
    Add    { path: PathBuf },
    Remove { name: String },
    List,
    Create { name: String },  
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
    // Run script with --manifest and parse JSON
    let out = Cmd::new("python3")           // interpreter call avoids chmod issues
        .arg(path)
        .arg("--manifest")
        .output()?;
    if !out.status.success() {
        return Err("plugin did not return valid manifest".into());
    }
    let manifest: Manifest = serde_json::from_slice(&out.stdout)?;

    // Copy script
    let dest_script = plugin_dir().join(&manifest.name);
    fs::copy(path, &dest_script)?;
    let mut perm = fs::metadata(&dest_script)?.permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&dest_script, perm)?;

    // Save manifest JSON
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

fn list_plugins() -> Result<(), IoError> {
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
    const TEMPLATE: &str = r#"#!/usr/bin/env python3
import sys, json

MANIFEST = {
    "name": "<<NAME>>",
    "description": "Describe what this plugin does",
    "version": "0.1.0",
    "commands": [
        { "name": "run",    "description": "Run the job" },
        { "name": "status", "description": "Show status" }
    ]
}

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

fn build_cli() -> Command {
    let mut cmd = Cli::command();  // static built-ins

    let trailing = Arg::new("args")
        .num_args(..)
        .trailing_var_arg(true)          // captures --flags etc. :contentReference[oaicite:1]{index=1}
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
