/*
 * Copyright (C) 2020 Red Hat, Inc.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use blsctl::bls::BLSEntry;
use blsctl::cmdline::CmdlineHandler;

use getopts::Matches;
use std::fs;
use std::io::{self, BufRead};
use std::process;

fn get_machine_id() -> std::io::Result<String> {
    let filename = "/etc/machine-id";

    let contents = fs::read_to_string(&filename)?;
    Ok(contents.trim().to_string())
}

fn add_bls_entry(
    title: &str,
    version: &str,
    linux: &str,
    initrd: &str,
    extra_initrd: &str,
    options: &str,
) {
    let machine_id = get_machine_id().unwrap_or_else(|e| {
        eprintln!("Could not read machine ID file: {}", e);
        process::exit(1);
    });

    let bls_filename = format!("{}-{}", &machine_id, &version);
    let blsentry = BLSEntry::create(&bls_filename).unwrap_or_else(|e| {
        eprintln!("Entry could not be created: {}", e);
        process::exit(1);
    });

    let mut initrds = vec![String::from(initrd)];

    if extra_initrd != "" {
        initrds.push(String::from(extra_initrd));
    }

    blsentry
        .set("title", &[String::from(title)])
        .expect("Could not set title key");
    blsentry
        .set("version",&[String::from(version)])
        .expect("Could not set version key");
    blsentry
        .set("linux", &[String::from(linux)])
        .expect("Could not set linux key");
    blsentry
        .set("initrd", &initrds)
        .expect("Could not set initrd key");
    blsentry
        .set("options", &[String::from(options)])
        .expect("Could not set options key");

    blsentry
        .set("grub_users", &[String::from("$grub_users")])
        .expect("Could not set grub_users key");
    blsentry
        .set("grub_arg", &[String::from("--unrestricted")])
        .expect("Could not set grub_arg key");
    blsentry
        .set("grub_class", &[String::from("kernel")])
        .expect("Could not set grub_class key");
}

fn handle_add_kernel(matches: &Matches) {
    if matches.opt_present("title") == false {
        eprintln!("The kernel title must be specified");
        process::exit(1);
    }

    let mut kernel = matches.opt_str("add-kernel").unwrap();
    let title = matches.opt_str("title").unwrap();

    let mut options = String::from("");
    if matches.opt_present("copy-default") == true {
        let blsentry = get_default_entry().unwrap_or_else(|e| {
            eprintln!("Default entry could not be obtained {}", e);
            process::exit(1);
        });

        let value = get_bls_value(&blsentry, "options");
        options = value[0].clone();
    }

    if matches.opt_present("args") == true {
        let args = matches.opt_str("args").unwrap();
        if options.is_empty() {
            options = args;
        } else {
            options = format!("{} {}", options, args);
        }
    }

    let mut version = String::from("");
    if kernel.contains("vmlinuz-") {
        let start = kernel.find("vmlinuz-").unwrap() + "vmlinuz-".len();
        if kernel.len() - start > 0 {
            version = kernel[start..kernel.len()].to_string();
        }
    }

    let prefix = get_prefix();

    let initrd_filename = format!("{}/initramfs-{}.img", &prefix, &version);
    let mut initrd = String::from(initrd_filename);
    if matches.opt_present("initrd") == true {
        initrd = matches.opt_str("initrd").unwrap();
    }

    let mut extra_initrd = String::from("");
    if matches.opt_present("extra-initrd") == true {
        extra_initrd = matches.opt_str("extra-initrd").unwrap();
    }

    if &prefix != "" {
        let start = kernel.find(&prefix).unwrap() + prefix.len();
        if kernel.len() - start > 0 {
            kernel = kernel[start..kernel.len()].to_string();
        }

        let start = initrd.find(&prefix).unwrap() + prefix.len();
        if initrd.len() - start > 0 {
            initrd = initrd[start..initrd.len()].to_string();
        }

        if extra_initrd != "" {
            let start = extra_initrd.find(&prefix).unwrap() + prefix.len();
            if extra_initrd.len() - start > 0 {
                initrd = extra_initrd[start..extra_initrd.len()].to_string();
            }
        }
    }

    add_bls_entry(&title, &version, &kernel, &initrd, &extra_initrd, &options);

    process::exit(0);
}

fn get_bls_value(entry: &BLSEntry, key: &str) -> Vec<String> {
    return match entry.get(key) {
        Ok(v) => v,
        Err(_e) => vec!["".to_string()],
    };
}

fn get_prefix() -> String {
    let filename = "/proc/self/mountinfo";
    let mount_point = "/boot";

    let file = fs::File::open(&filename).unwrap_or_else(|e| {
        eprintln!("Could not open mountinfo: {}", e);
        process::exit(1);
    });

    let lines = io::BufReader::new(file).lines();

    for line in lines {
        if let Ok(line) = line {
            let columns: Vec<&str> = line.split(' ').collect();
            if &columns[4] == &mount_point {
                return String::from(mount_point);
            }
        }
    }

    String::from("")
}

fn get_default_index() -> usize {
    0
}

fn separate_root_param(options: &str) -> (String, String) {
    let mut root = String::new();
    let mut args = String::new();

    for option in options.split(" ") {
        if option.starts_with("root=") {
            root = (&option["root=".len()..option.len()]).to_string();
        } else {
            args.push_str(option);
            args.push_str(" ");
        }
    }
    args.pop();

    (args, root)
}

fn get_default_entry() -> Result<BLSEntry, String> {
    let mut index = 0;
    let default = get_default_index();

    let entries = BLSEntry::get_bls_entries().unwrap_or_else(|e| {
        eprintln!("Could not read bootloader entries from directory: {}", e);
        process::exit(1);
    });

    for entry in entries {
        let blsentry = BLSEntry::new(&entry).unwrap_or_else(|e| {
            eprintln!("Could not read bootloader entry {}: {}", entry, e);
            process::exit(1);
        });

        if index == default {
            return Ok(blsentry);
        }

        index = index + 1;
    }

    Err(String::from("Could not get default entry"))
}

fn get_bls_list(param: &str) -> Vec<(usize, BLSEntry)> {
    let mut index = 0;
    let default = get_default_index();
    let prefix = get_prefix();

    let entries = BLSEntry::get_bls_entries().unwrap_or_else(|e| {
        eprintln!(
            "ERROR: could not read bootloader entries from directory: {}",
            e
        );
        process::exit(1);
    });

    let mut list = Vec::new();

    for entry in entries {
        let blsentry = BLSEntry::new(&entry).unwrap_or_else(|e| {
            eprintln!("Could not read bootloader entry {}: {}", entry, e);
            process::exit(1);
        });

        let title = get_bls_value(&blsentry, "title");
        let linux = get_bls_value(&blsentry, "linux");

        if param == "ALL"
            || (param == "DEFAULT" && index == default)
            || param == index.to_string()
            || param == format!("{}{}", &prefix, linux[0])
            || param == format!("TITLE={}", &title[0])
        {
            list.push((index, blsentry));
        }

        index = index + 1;
    }

    list
}

fn print_bls_entries(entries: Vec<(usize, BLSEntry)>) {
    let prefix = get_prefix();

    for (index, bls) in entries {
        let title = get_bls_value(&bls, "title");
        let linux = get_bls_value(&bls, "linux");
        let initrd = get_bls_value(&bls, "initrd");
        let options = get_bls_value(&bls, "options");

        let id = &bls.name[..&bls.name.len() - ".conf".len()];

        let options = separate_root_param(&options[0]);

        println!("index={}", index);
        println!("kernel=\"{}{}\"", prefix, linux[0]);
        println!("args=\"{}\"", options.0);
        println!("root=\"{}\"", options.1);
        println!("initrd=\"{}{}\"", prefix, initrd[0]);
        if initrd.len() > 1 {
          println!("initrd=\"{}{}\"", prefix, initrd[1]);
        }
        println!("title=\"{}\"", title[0]);
        println!("id=\"{}\"", id);
    }
}

fn handle_info(matches: &Matches) {
    let info = matches.opt_str("info").unwrap();

    let entries = get_bls_list(&info);

    print_bls_entries(entries);

    process::exit(0);
}

fn unset_default_bls() {}

fn remove_bls_entries(entries: Vec<(usize, BLSEntry)>) {
    let default = get_default_index();

    for (index, bls) in entries {
        if index == default {
            unset_default_bls();
        }
        bls.delete().unwrap_or_else(|e| {
            eprintln!(
                "ERROR: could not delete bootloader entry from directory: {}",
                e
            );
            process::exit(1);
        })
    }
}

fn handle_remove_kernel(matches: &Matches) {
    let remove_kernel = matches.opt_str("remove-kernel").unwrap();

    let entries = get_bls_list(&remove_kernel);

    remove_bls_entries(entries);

    process::exit(0);
}

fn update_bls_entries(entries: &mut Vec<(usize, BLSEntry)>, remove_args: &str, args: &str) {
    let remove_args = remove_args
        .split(" ")
        .map(|arg| String::from(arg))
        .collect::<Vec<String>>();

    let args = args
        .split(" ")
        .map(|arg| String::from(arg))
        .collect::<Vec<String>>();

    for (_index, bls) in entries {
        bls.cmdline_remove(&remove_args)
            .expect("Could not remove arguments for BLS");
        bls.cmdline_set(&args)
            .expect("Could not set arguments for BLS");
    }
}

fn handle_update_kernel(matches: &Matches) {
    if matches.opt_present("args") == false && matches.opt_present("remove-args") == false {
        eprintln!("Kernel arguments to add or remove were not specified");
        process::exit(1);
    }

    let kernel = matches.opt_str("update-kernel").unwrap();
    let mut entries = get_bls_list(&kernel);

    let args = match matches.opt_str("args") {
        Some(v) => v,
        _ => String::from(""),
    };

    let remove_args = match matches.opt_str("remove-args") {
        Some(v) => v,
        _ => String::from(""),
    };

    update_bls_entries(&mut entries, &remove_args, &args);

    process::exit(0);
}

pub fn run(matches: &Matches) {
    if matches.opt_present("add-kernel") {
        handle_add_kernel(matches);
    }

    if matches.opt_present("info") {
        handle_info(matches);
    }

    if matches.opt_present("remove-kernel") {
        handle_remove_kernel(matches);
    }

    if matches.opt_present("update-kernel") {
        handle_update_kernel(matches);
    }
}
