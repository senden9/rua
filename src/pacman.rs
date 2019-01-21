use crate::libalpm::Alpm;
use crate::util;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub fn is_package_installed(alpm: &Alpm, name: &str) -> bool {
    alpm.local_db()
        .find_satisfier(name)
        .expect("Failed to access libalpm.find_satisfier")
        .map_or(false, |sat| sat.install_date().is_some())
}

pub fn is_package_installable(alpm: &Alpm, name: &str) -> bool {
    alpm.find_satisfier(name)
        .expect("Failed to access libalpm.find_satisfier")
        .is_some()
}

pub fn get_repository_list() -> Vec<String> {
    let cmd = Command::new("pacman-conf")
        .arg("--repo-list")
        .output()
        .expect("cannot get repository list: pacman-conf --repo-list");
    let output = String::from_utf8(cmd.stdout)
        .expect("Failed to get repo list from `pacman-conf --repo-list`");
    output.lines().map(|s| s.to_owned()).collect()
}

fn ensure_packages_installed(
    mut packages: HashMap<String, PathBuf>,
    base_args: &[&str],
    alpm: &Alpm,
) {
    while !packages.is_empty() {
        {
            let mut list = packages
                .iter()
                .map(|(_name, path)| {
                    path.to_str().unwrap_or_else(|| {
                        panic!("{}:{} cannot parse package name", file!(), line!())
                    })
                })
                .collect::<Vec<_>>();
            list.sort_unstable();
            eprintln!("Packages need to be installed:");
            eprintln!(
                "\n    pacman {} --needed {}\n",
                base_args.join(" "),
                list.join(" ")
            );
            eprint!("Enter S to `sudo` install it, or install manually and press M when done. ");
            let string = util::console_get_line();
            if string == "s" {
                Command::new("sudo")
                    .arg("pacman")
                    .args(base_args)
                    .arg("--needed")
                    .args(&list)
                    .status()
                    .ok();
            } else if string == "m" {
                break;
            }
        }
        packages.retain(|name, _| !is_package_installed(alpm, name));
    }
}

pub fn ensure_aur_packages_installed(
    packages: HashMap<String, PathBuf>,
    is_dependency: bool,
    alpm: &Alpm,
) {
    if is_dependency {
        ensure_packages_installed(packages, &["-U", "--asdeps"], alpm);
    } else {
        ensure_packages_installed(packages, &["-U"], alpm);
    }
}

pub fn ensure_pacman_packages_installed(packages: HashSet<String>, alpm_db: &Alpm) {
    let mut map: HashMap<String, PathBuf> = HashMap::new();
    for package in packages {
        let path = Path::new(&package).to_path_buf();
        map.insert(package, path);
    }
    ensure_packages_installed(map, &["-S", "--asdeps"], alpm_db);
}

// let's commit this to git and clean up later, so it'll stay in history (if will ever be needed)
//pub fn is_package_installable(package: &str) -> bool {
//	Command::new("pacman").arg("-Sddp").arg(&package)
//		.stdout(Stdio::null()).stderr(Stdio::null()).status()
//		.expect(&format!("Failed to determine if package {} is installable", package))
//		.success()
//}
