use std::path::PathBuf;

use clap::{Parser, Subcommand};
use enum_map::Enum;

use strum::IntoEnumIterator; // 0.17.1
use strum_macros::{Display, EnumIter}; // 0.17.1

use git2::Repository; // 0.13.7

const NEOVIM_CONFIGURATION_REPO_URL: &str = "https://gitlab.com/bjk2k/configurations-neovim.git";
const TMUX_CONFIGURATION_REPO_URL: &str = "https://gitlab.com/bjk2k/configurations-tmux.git";
const PUBLIC_KEYS_REPO_URL: &str = "";

// function to check if path is directory and writable
fn path_validator(v: &str) -> Result<String, String> {
    if std::path::Path::new(v).is_dir() {
        if std::fs::metadata(std::path::Path::new(v))
            .unwrap()
            .permissions()
            .readonly()
        {
            return Err(String::from("Directory is not writable"));
        }
        return Ok(String::from(v));
    }
    return Err(String::from("Path is not a directory"));
}

/// A basic cli tool
#[derive(Parser, Debug)]
#[clap(author="bjk2k", about="Invite the red pandas.", version = None)]
struct Args {
    #[clap(subcommand)]
    subcmd: Commands,
}

// sub command for installation
#[derive(Subcommand, Debug)]
enum Commands {
    Install {
        #[arg(value_name = "INSTALL_DIR", value_parser=path_validator, help="Directory to install into")]
        directory: String,

        #[arg(short, long, help="Features to install", num_args=1..)]
        features: Vec<Feature>,
    },
    FullInstall {
        #[arg(value_name = "INSTALL_DIR", value_parser=path_validator, help="Directory to install into")]
        directory: String,
    },
    List,
}

#[derive(clap::ValueEnum, Clone, Debug, Enum, EnumIter, Display)]
enum Feature {
    #[clap(name = "zsh")]
    ZSH,
    #[clap(name = "neovim")]
    NeoVIM,
    #[clap(name = "pubkeys")]
    PublicKeys,
    #[clap(name = "tmux")]
    TMUX,
}

impl Feature {
    fn install(&self, directory: &PathBuf) {
        match self {
            Feature::ZSH => install_zsh(directory),
            Feature::NeoVIM => install_neovim(directory),
            Feature::PublicKeys => install_public_keys(directory),
            Feature::TMUX => install_tmux(directory),
        }
    }
}

fn install_neovim_dependencies(base_directory: &PathBuf, custom_neovim_config_dir: &PathBuf) {
    // path to configurations-neovim >> vscode >> nvim
    let path_to_configuration = custom_neovim_config_dir.join("vscode").join("nvim");

    // trigger install script
    println!("    |- Triggering install script for neovim dependencies @ {}", custom_neovim_config_dir.join("setup.sh").display());
    let mut cmd = std::process::Command::new("bash");
    cmd.arg(custom_neovim_config_dir.join("setup.sh"))
        .arg(base_directory);
    let output = cmd.output().expect("failed to execute setup.sh for configurations-neovim.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));

    // link neovim configuration
    println!("    |- Linking neovim configuration");
    let mut cmd = std::process::Command::new("ln");
    cmd.arg("-s").arg(path_to_configuration).arg(
        std::path::Path::new(&std::env::var("HOME").unwrap())
            .join(".config")
            .join("nvim"),
    );
    let output = cmd.output().expect("failed to link custom neovim configuration.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));
}

fn install_neovim(base_directory: &PathBuf) {
    let neovim_config_dir = base_directory.join("configurations-neovim");

    println!(
        " |- Downloading neovim configuration into <{}>",
        neovim_config_dir.display()
    );

    // clone configuration repository
    let _repo = match Repository::clone(NEOVIM_CONFIGURATION_REPO_URL, &neovim_config_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    // install dependencies
    install_neovim_dependencies(base_directory, &neovim_config_dir);
}

fn install_tmux(base_directory: &PathBuf) {
    let tmux_config_dir = base_directory.join("configurations-tmux");
    println!(
        " |- Downloading tmux configuration into <{}>",
        tmux_config_dir.display()
    );

    // clone configuration repository
    let _repo = match Repository::clone(TMUX_CONFIGURATION_REPO_URL, &tmux_config_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    println!("    |- Linking neovim configuration");
    let mut cmd = std::process::Command::new("ln");
    cmd.arg("-s").arg(tmux_config_dir).arg(
        std::path::Path::new(&std::env::var("HOME").unwrap())
            .join(".config")
            .join("nvim"),
    );
    let output = cmd.output().expect("failed to link custom neovim configuration.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));
}

fn install_public_keys(base_directory: &PathBuf) {
    let neovim_config_dir = base_directory.join("public-keys");

    println!(
        " |- Downloading public keys into <{}>",
        neovim_config_dir.display()
    );

    let _repo = match Repository::clone(PUBLIC_KEYS_REPO_URL, &neovim_config_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
}

fn install_zsh(base_directory: &PathBuf) {
    let oh_my_zsh_dir = base_directory.join("oh-my-zsh");
    let _zsh_dotfile_dir = base_directory.join("zsh-dotfiles");
   
    let mut cmd = std::process::Command::new("wget");
    cmd.arg("https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh");

    let output = cmd.output().expect("failed to download oh-my-zsh install script.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));

    let mut cmd = std::process::Command::new("sh");
    cmd.env("ZSH", oh_my_zsh_dir).arg("install.sh").arg("--unattended");

    let output = cmd.output().expect("failed to install oh-my-zsh via install script.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));

    // remove install.sh if exists

    if std::path::Path::new("install.sh").exists() {
        std::fs::remove_file("install.sh").expect("failed to remove install.sh not present anymore!");
            
    }
}

fn install(directory: &String, features: &Vec<Feature>) {
    // create installation repository
    let install_dir = std::path::Path::new(&directory).join("red-panda-hollow");
    println!(
        "[O] Inviting some red pandas into the <{}> directory",
        install_dir.display()
    );
    println!("[O] Installing features: {:?}", features);
    for feature in features {
        feature.install(&install_dir);
    }
}

fn list() {
    println!("[O] Listing all features ...");
    for feature in Feature::iter() {
        println!("    |- {}", feature);
    }
}

fn main() {
    let args = Args::parse();
    match args.subcmd {
        Commands::Install {
            directory,
            features,
        } => {
            install(&directory, &features);
        }
        Commands::FullInstall { directory } => {
            install(
                &directory,
                Feature::iter().collect::<Vec<Feature>>().as_ref(),
            );
        }
        Commands::List => {
            list();
        }
    }
}
