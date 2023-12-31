use std::{path::PathBuf, io::Write};

use clap::{Parser, Subcommand};
use enum_map::Enum;

use strum::IntoEnumIterator; // 0.17.1
use strum_macros::{Display, EnumIter}; // 0.17.1

use git2::Repository; // 0.13.7

const NEOVIM_CONFIGURATION_REPO_URL: &str = "https://gitlab.com/bjk2k/configurations-neovim.git";
const TMUX_CONFIGURATION_REPO_URL: &str = "https://gitlab.com/bjk2k/configurations-tmux.git";
const DOTFILES_REPO_URL: &str = "https://gitlab.com/bjk2k/dotfiles-red-panda.git";
const PUBLIC_KEYS_REPO_URL: &str = "";

const ASCII_HEADER: &str = "                             ███                             
                      █      █           █                   
                       ██    █  █       █                    
                       █         ██████  █                   
                                    █   █                    
  ██    ███                             █          ██    ██  
█           ██          █              █       ██            
         ██    █        █              █     █     █         
█     █████  █   ██   ███             █    █   █  █████     █
    █    █████ █   ███████  █        █████   █ █████    █    
        ██   ██  █ ███████████████████████ █  ██   ██        
█              █ ███████████████████████████ █      █   █   █
      █      █ ███████████████████████████████        █      
██         ██████████████████████████████████████          ██
█     ███████████████████████████████████████████████████   █
█ █     █ █████████████████████████████████████████ █     █ █
 █   █   ████████████████  ███████   ███████████████   ██  █ 
   ██   ███████████████     ██████    ███████████████   ██   
 ██    ██████████████████   ██████  ██████████████████    ██ 
██ ██ ██████████████  ██ ███████████ ███  ████████████████ ██
    ███████     ███ ██████ █     ██ ██████ ██     ███████    
   ███████     █████ ████ █        █ ███  ████      ██████   
     ████      █████████             ██████████     ████     
     ████     ███████                    ██████     ████     
      ███     █████  █     ████████     █ ██████    ████     
     █████    █████       █████████  █     █████   █████     
     ██████   █████        ████████       ███ █   ██████     
     ██ █████  ██████         █          █████  █████ ██     
       ████████  █████ ██  ██   ██  ███ ████  ████████       
           ████████                        ███████           
               █████████             █████████               
                    █████████    ████████                    
                                                        @bjk2k";
                                                       
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
    #[clap(name = "secretkeys")]
    SecretKeys,
}

impl Feature {
    fn install(&self, directory: &PathBuf, dotfiles_directory: &PathBuf) {
        match self {
            Feature::ZSH => install_zsh(directory, dotfiles_directory),
            Feature::NeoVIM => install_neovim(directory, dotfiles_directory),
            Feature::PublicKeys => install_public_keys(directory, dotfiles_directory),
            Feature::TMUX => install_tmux(directory, dotfiles_directory),
            Feature::SecretKeys => install_secretkeys(directory, dotfiles_directory),
        }
    }
}

fn install_secretkeys(_base_directory: &PathBuf, _dotfiles_directory: &PathBuf) {
    panic!("Secret keys not implemented yet!");
}

fn install_neovim_dependencies(base_directory: &PathBuf, custom_neovim_config_dir: &PathBuf) {
    // path to configurations-neovim >> vscode >> nvim
    let path_to_configuration = custom_neovim_config_dir;

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

fn install_neovim(base_directory: &PathBuf, _dotfiles_directory: &PathBuf) {
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

fn install_tmux(base_directory: &PathBuf, _dotfiles_directory: &PathBuf) {
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
            .join("tmux"),
    );
    let output = cmd.output().expect("failed to link custom neovim configuration.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));
}

fn install_public_keys(base_directory: &PathBuf, _dotfiles_directory: &PathBuf) {
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

fn reset_dotfile_package(dotfiles_directory: &PathBuf, stow_package: &String) {
    let subdirectory = dotfiles_directory.join(stow_package);
    
    // git resore -- directory

    let mut cmd = std::process::Command::new("git");
    cmd.arg("restore").arg("--").arg(subdirectory);

    println!("    |- Restoring dotfile package {}", stow_package);
    let output = cmd.output().expect("failed to restore dotfile package.");
    println!("       {}", String::from_utf8_lossy(&output.stdout));
    
}

fn install_zsh(_base_directory: &PathBuf, dotfiles_directory: &PathBuf) {
  
   let mut cmd = std::process::Command::new("wget");
    cmd.arg("https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh");

    let output = cmd.output().expect("failed to download oh-my-zsh install script.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));

    // run install script inside zsh

    let mut cmd = std::process::Command::new("zsh");
    cmd.arg("install.sh");

    let output = cmd.output().expect("failed to install oh-my-zsh via install script.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));

    // Clean up - remove install.sh if exists
    if std::path::Path::new("install.sh").exists() {
        std::fs::remove_file("install.sh").expect("failed to remove install.sh not present anymore!");
    }
    
    // Clean up - back up new .zshrc
    if std::path::Path::new(".zshrc").exists() {
        std::fs::rename(".zshrc", ".zshrc.bak").expect("failed to back up .zshrc");
    }

    // Clean up - rename .zshrc.pre-oh-my-zsh to .zshrc
    if std::path::Path::new(".zshrc.pre-oh-my-zsh").exists() {
        std::fs::rename(".zshrc.pre-oh-my-zsh", ".zshrc").expect("failed to rename .zshrc.pre-oh-my-zsh to .zshrc");
    } else {
        println!("    |- .zshrc.pre-oh-my-zsh not present, skipping");
        std::fs::rename(".zshrc.bak", ".zshrc").expect("failed to restore back up .zshrc");
    }

    reset_dotfile_package(dotfiles_directory, &String::from("zsh"));
}

fn setup_dotfiles(base_directory: &PathBuf, dotfiles_directory: &PathBuf) {
    
    // check if dotfiles directory exists and is repo if not clone it

    let _repo = match Repository::open(dotfiles_directory) {
        Ok(_repo) => {
            println!("    |- Dotfiles directory present, removing directory and cloning again ...");
            
            // TODO: do proper pull instead of removing directory and cloning again
            std::fs::remove_dir_all(dotfiles_directory).expect("failed to remove dotfiles directory");
            
            match Repository::clone(DOTFILES_REPO_URL, dotfiles_directory) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            }
        },
        Err(_) => {
            println!("    |- Dotfiles directory not present, cloning");
            match Repository::clone(DOTFILES_REPO_URL, dotfiles_directory) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            }
        }
    };
    
    println!("    |- Setting up dotfiles");

    // trigger install script
    let mut cmd = std::process::Command::new("bash");
    cmd.current_dir(dotfiles_directory);
    cmd.arg(dotfiles_directory.join("install.sh"));
    
    println!("    |- Triggering install script for dotfiles @ {}", dotfiles_directory.join("install.sh").display());
    let output = cmd.output().expect("failed to execute install.sh for dotfiles.");
    println!("    |- {}", String::from_utf8_lossy(&output.stdout));

    // create or overide ~/.red_panda_setup.sh
    
    let mut file = std::fs::OpenOptions::new().write(true).truncate(true).open(
        std::path::Path::new(&std::env::var("HOME").unwrap())
            .join(".red_panda_setup.sh")
            ).unwrap();
    file.write_all(format!("export RED_PANDA_HOME={}\n", base_directory.display()).as_bytes()).expect("failed to write to ~/.red_panda_setup.sh");    
    // set env variable RED_PANDA_HOME to base directory

    // git restore all subdirectories in dotfiles repo -- stow packages
    
    for entry in std::fs::read_dir(dotfiles_directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            reset_dotfile_package(dotfiles_directory, &path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
}

fn install(directory: &String, features: &Vec<Feature>) {
    // create installation repository
    let install_dir = std::path::Path::new(&directory).join("red-panda-hollow");
    let dotfile_dir = std::path::Path::new(&directory).join(".red-panda-dotfiles");
    
    println!("[O] Setting up some red pandas dotfiles in the <{}> directory", dotfile_dir.display()); 
    setup_dotfiles(&install_dir, &dotfile_dir);
    
    println!(
        "[O] Inviting some red pandas into the <{}> directory",
        install_dir.display()
    );
    println!("[O] Installing features: {:?}", features);
    for feature in features {
        feature.install(&install_dir, &dotfile_dir);
    }
}

fn list() {
    println!("[O] Listing all features ...");
    for feature in Feature::iter() {
        println!("    |- {}", feature);
    }
}

// function to check if all dependencies are met
fn check_dependencies() -> Result<(), &'static str> {
    let mut cmd = std::process::Command::new("stow");
    cmd.arg("--version");

    if cmd.status().is_err() {
        return Err("Stow is not installed and can not be installed! Install it!");
    }

    let mut cmd = std::process::Command::new("zsh");
    cmd.arg("--version");

    if cmd.status().is_err() {
        return Err("ZSH is not installed and can not be installed! Install it!");
    }

    let mut cmd = std::process::Command::new("wget");
    cmd.arg("--version");

    if cmd.status().is_err() {
        return Err("wget is not installed and can not be installed! Install it!");
    }

    Ok(())
}

fn main() {
    
    println!("{}", ASCII_HEADER);
    let args = Args::parse();
    
    match check_dependencies() {
        Ok(_) => println!("[O] \n |- dependencies ✔"),
        Err(e) => panic!("failed to check dependencies: {}", e),
    }
    
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
