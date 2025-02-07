# Seneca Solver

A simple program that automatically solves your assignments in Seneca (and also gets you a bunch of free XP 🔥). Note that this is still early in development and may have bugs.

It should work on most assignments but probably will not complete every single one. This is due to the fact that not all question types are supported at the moment, although this is currently being worked on. In addition, it **will not work** on the AI-marked "Exam Questions" at the end of assignments. There is very little that can be done about that since that's marked on Seneca's servers.

## Features

- Automatically solves assignments
- Works on most question types
- Nice and intuitive terminal interface
- Auto mode: solves all assignments
- Semi-automatic mode: Select a specific assignment to solve
- Manual mode: solve a specific sub-section of an assignment

## How to set up

### Method 1: Executable (recommended)

> [!NOTE]
> If you are using Windows, you may get a warning from Windows Defender SmartScreen when running the executable. This is because the executable is not signed. You can safely ignore this warning by clicking "More info" and then "Run anyway".

Download the executable from the links below. Alternatively, download from the [releases page](https://github.com/ArcaEge/seneca-solver/releases/latest), making sure to download the latest release.

| System             | File                               |
| ------------------ | ---------------------------------- |
| 64-bit x86 Windows | [`seneca-solver_windows-x86_64.exe`](https://github.com/ArcaEge/seneca-solver/releases/latest/download/seneca-solver_windows-x86_64.exe) |
| 64-bit x86 macOS   | [`seneca-solver_macos-x86_64`](https://github.com/ArcaEge/seneca-solver/releases/latest/download/seneca-solver_macos-x86_64) |
| M-series ARM macOS | [`seneca-solver_macos-aarch64`](https://github.com/ArcaEge/seneca-solver/releases/latest/download/seneca-solver_macos-aarch64) |
| 64-bit x86 Linux   | [`seneca-solver_linux-x86_64`](https://github.com/ArcaEge/seneca-solver/releases/latest/download/seneca-solver_linux-x86_64) |

If on a UNIX-based system (i.e. Linux or macOS), add the execute permission to the executable using `chmod +x <name-of-executable>`. Move on to the "Usage" section below. If your OS/system architecture is unsupported, use Method 2.

### Method 2: Compile it yourself

#### Requirements

You must have the following installed:

- Cargo (any relatively new version will do)

#### Steps

1. Run `git clone https://github.com/ArcaEge/seneca-solver.git` to clone the repository to somewhere convenient.

1. Open the repository directory in a terminal and run `cargo build --release`. This will compile the program and place the executable in `target/release/seneca-solver`.

## How to use

1. You need to generate a `seneca-solver-keys.json` file that contains your API key and refresh token. Here's how to do that:
    1. Go to the Seneca dashboard page in your browser and open the inspect menu.
    1. Navigate to the Console tab.
    1. Paste in the contents of the [key_extractor.js](key_extractor.js) file into the console and press enter to run it. This will then generate and download the `seneca-solver-keys.json` file. All you then need to do is copy this file to the same folder as the `seneca-solver` executable.

> [!NOTE]
> Most browsers will show a warning like this if you haven't pasted anything into a browser console before:
> ![Warning: Don’t paste code into the DevTools Console that you don’t understand or haven’t reviewed yourself. This could allow attackers to steal your identity or take control of your computer. Please type ‘allow pasting’ below and hit Enter to allow pasting.](docs/images/allow_pasting.png)
>
> To bypass this, simply type in `allow pasting` and press enter as it says in the message.

> [!WARNING]
> Do not share this generated file with anyone, otherwise they can access everything in your Seneca account.

2. Run the executable. If you ran it from your file manager, it should've spawned a terminal window.

1. Your assignments will be printed to the terminal. Use the arrow keys to navigate between them. Press `Enter` to start solving an assignment.

1. The script should now start solving the assignment, which may take 5 or more minutes depending on its length. **It will likely not manage to solve all sections of the assignment** - I believe this is due to Seneca's servers but I am looking for workarounds. Try running it a few more times (the order is shuffled every time in the code) or use the `Custom (URL)` mode and manually paste in the URLs of the sections. Also note that your access key will expire every hour, meaning you will need to repeat the process to obtain the key every hour. Using refresh tokens instead of access keys is on the To Do list.

## Troubleshooting/common errors

### Rate limit

``` rust
reqwest::Error { kind: Status(403) ... }
```

You have been rate-limited by Seneca, wait for a few minutes and try again.

## Advanced use

### 🚜 Docker image for generating XP

```sh
docker run -v /path/to/seneca-solver-keys.json:/seneca-solver-keys.json ghcr.io/arcaege/seneca-solver:main
```

## Disclaimer

This program is not affiliated with Seneca Learning in any way - it's a third-party tool that impersonates the Seneca frontend. **Using this tool might be against Seneca's Terms of Service.** Use it at your own risk - I'm not responsible if you get in trouble for using this tool.

In legal terms: The author of this program is not responsible for any consequences that may arise from using this tool.
