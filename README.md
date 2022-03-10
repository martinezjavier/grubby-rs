# grubby

[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://img.shields.io/badge/License-GPL%20v2-blue.svg)

A Rust reimplementation of the
[grubby](https://src.fedoraproject.org/rpms/grubby/blob/rawhide/f/grubby-bls)
command line tool that updates and displays information about boot entries
for the grub2 and zipl bootloaders.

It is primarily designed to be used from scripts which install new kernels
and need to find information about the current boot environment.

NOTE: This project is still a work in progress and not ready for production.
