/*
 * Copyright (C) 2020 Red Hat, Inc.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

extern crate getopts;
use getopts::Options;
use std::env;
use std::process;

fn print_usage(program: &str, opts: &Options) {
    let usage = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&usage));
}

fn fill_opts() -> Options {
    let mut opts = Options::new();

    opts.optopt(
        "",
        "add-kernel",
        "add an entry for the specified kernel",
        "kernel-path",
    );
    opts.optopt(
        "",
        "args",
        "default arguments for the new kernel or new arguments for kernel being updated",
        "args",
    );
    opts.optflag(
        "",
        "bad-image-okay",
        "don't sanity check images in boot entries (for testing only)",
    );
    opts.optopt(
        "",
        "config-file",
        "path to grub config file to update",
        "path",
    );
    opts.optflag(
        "",
        "copy-default",
        "use the default boot entry as a template for the new entry being added; if the default is not a linux image, or if the kernel referenced by the default image does not exist, the first linux entry whose kernel does exist is used as the template"
    );
    opts.optflag(
        "",
        "default-kernel",
        "display the path of the default kernel",
    );
    opts.optflag(
        "",
        "default-index",
        "display the index of the default kernel",
    );
    opts.optflag(
        "",
        "default-title",
        "display the title of the default kernel",
    );
    opts.optopt("", "env", "path for grub2 environment block file", "path");
    opts.optflag("", "grub2", "configure grub2 bootloader");
    opts.optopt(
        "",
        "info",
        "display boot information for specified kernel",
        "kernel-path",
    );
    opts.optopt(
        "",
        "initrd",
        "initrd image for the new kernel",
        "initrd-path",
    );
    opts.optopt(
        "",
        "extra-initrd",
        "auxiliary initrd image for things other than the new kernel",
        "initrd-path",
    );
    opts.optflag(
        "",
        "make-default",
        "make the newly added entry the default boot entry",
    );
    opts.optopt("", "remove-args", "remove kernel arguments", "args");
    opts.optopt(
        "",
        "remove-kernel",
        "remove all entries for the specified kernel",
        "kernel-path",
    );
    opts.optopt(
        "",
        "set-default",
        "make the first entry referencing the specified kernel the default",
        "kernel-path",
    );
    opts.optopt(
        "",
        "set-default-index",
        "make the given entry index the default entry",
        "entry-index",
    );
    opts.optopt(
        "",
        "title",
        "title to use for the new kernel entry",
        "entry-title",
    );
    opts.optopt(
        "",
        "update-kernel",
        "title to use for the new kernel entry",
        "kernel-path",
    );
    opts.optflag("", "zipl", "configure zipl bootloader");
    opts.optopt(
        "",
        "bls-directory",
        "path to directory containing the BootLoaderSpec fragment files",
        "path",
    );
    opts.optflag(
        "",
        "no-etc-grub-update",
        "don't update the GRUB_CMDLINE_LINUX variable in /etc/default/grub",
    );
    opts.optflag("", "help", "show this help");

    opts
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let opts = fill_opts();

    let matches = opts.parse(&args[1..]).unwrap_or_else(|e| {
        eprintln!("{}: {}", program, e);
        process::exit(1);
    });

    if matches.opt_present("help") || args.len() == 1 {
        print_usage(&program, &opts);
        process::exit(0);
    }

    grubby::run(&matches);

    process::exit(0);
}
