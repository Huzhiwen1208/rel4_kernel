#!/usr/bin/env python3
#
# Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
#
# SPDX-License-Identifier: BSD-2-Clause
#

import subprocess
import sys
import argparse
import time
import os
import shutil
from pygments import highlight
from pygments.lexers import BashLexer
from pygments.formatters import TerminalFormatter

build_dir = "./build"

def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('-b', '--baseline', dest="baseline", action="store_true",
                        help="baseline switch")

    parser.add_argument('-u', '--uintr', dest="uintr_enable", action="store_true",
                            help="uintr support")

    parser.add_argument('-c', '--cpu', dest="cpu_nums", type=int,
                        help="kernel & qemu cpu nums", default=1)

    parser.add_argument('-i', '--install', dest="install", action="store_true",
                            help="install kernel & set env")

    parser.add_argument('-r', '--rt', dest="rust_test", action="store_true",
                        help="run rust root task demo")

    parser.add_argument('-f', '--fpga_board', dest="fpga_net_test", action="store_true",
                        help="run rust root task demo")

    parser.add_argument('-p', '--push_remote', dest="push_remote", type=str,
                        help="push remote machine if u are docker container", default="")
    args = parser.parse_args()
    return args

def exec_shell(shell_command):
    ret_code = os.system(shell_command)
    return ret_code == 0

def clean_config():
    shell_command = "cd ../kernel && git checkout master"
    exec_shell(shell_command)
    base_test_setting_path = os.path.abspath("../projects/sel4test/easy-settings.cmake")
    shell_command = "cd .. && ln -snf " + base_test_setting_path + " ./easy-settings.cmake"
    exec_shell(shell_command)


def install_kernel():
    shell_command = "cd ../kernel && cmake -DCROSS_COMPILER_PREFIX=riscv64-unknown-linux-gnu-" + \
                    " -DCMAKE_TOOLCHAIN_FILE=gcc.cmake" + " -DCMAKE_INSTALL_PREFIX=install" + \
                    " -C ./kernel-settings.cmake" + " -G Ninja" + " -S . -B build" + \
                    " && ninja -C build all && ninja -C build install"
    exec_shell(shell_command)

def push_remote(remote_addr):
    shell_command = "ssh ctrlz@" + remote_addr + " \"bash -s\" < ./push_remote.sh"
    exec_shell(shell_command)

if __name__ == "__main__":
    args = parse_args()
    clean_config()
    progname = sys.argv[0]

    if not os.path.exists(build_dir):
#         shutil.rmtree(build_dir)
        os.makedirs(build_dir)
#     os.makedirs(build_dir)
    if args.rust_test == True:
        rt_setting_path = os.path.abspath("../projects/rust-root-task-demo/easy-settings.cmake")
        shell_command = "cd .. && ln -snf " + rt_setting_path + " ./easy-settings.cmake"
        if not exec_shell(shell_command):
            clean_config()
            sys.exit(-1)
    if args.baseline == True:
        shell_command = "cd ../kernel && git checkout baseline"
        if not exec_shell(shell_command):
            clean_config()
            sys.exit(-1)
    else:
        shell_command = "cargo build --release --target riscv64imac-unknown-none-elf"
        if args.cpu_nums > 1:
            shell_command += " --features ENABLE_SMP"
        if args.uintr_enable:
            shell_command += " --features ENABLE_UINTC"
        if args.fpga_net_test:
            shell_command += " --features board_lrv"
        else:
            shell_command += " --features board_qemu"
        if not exec_shell(shell_command):
            clean_config()
            sys.exit(-1)
    if args.install:
        install_kernel()
    else:
        shell_command = "cd ./build && ../../init-build.sh  -DPLATFORM=spike -DSIMULATION=TRUE"
        if args.cpu_nums > 1:
            shell_command += " -DSMP=TRUE"
        if args.uintr_enable:
            shell_command += " -DUINTR=TRUE"
        shell_command += " && ninja -v"
        if not exec_shell(shell_command):
            clean_config()
            sys.exit(-1)
        clean_config()

    if args.push_remote != "":
        push_remote(args.push_remote)


