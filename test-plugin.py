#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# dependencies = [
#     ///add you dependencies here
# ]
# ///
import sys, json, subprocess

MANIFEST = {
    "name": "test-plugin",
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
    print("Running test-plugin with", args)

def status(args):
    print("test-plugin status:", args)

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
