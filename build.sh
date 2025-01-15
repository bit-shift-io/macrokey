#!/bin/bash

function main {
    # loop args
    if [[ $# -ne 0 ]] ; then
        for var in "$@" ; do
            eval $var
        done
        exit 1
    fi
    
    # menu
    while true; do
    read -n 1 -p "
    config
    ===================
    1) Run Release
    2) Build Release
    
    3) Build Debug
    4) Run Debug
    
    6) Install ROOT Service
    7) Install USER Service
    
    *) Any key to exit
    :" ans;
    reset
    case $ans in
        1) fn_run ;;
        2) fn_build ;;
        3) fn_build_debug ;;
        4) fn_run_debug ;;
        6) fn_root_service ;;
        7) fn_user_service ;;
        *) $SHELL ;;
    esac
    done
}


function fn_user_service {
    sudo usermod -a -G input $USER
    newgrp input # login to group

    echo "added to input group"
    
    mkdir -p $HOME/.config/systemd/user/
    DIR="$( cd "$( dirname "$0" )" && pwd )"

tee $HOME/.config/systemd/user/macrokey.service > /dev/null << EOL 
[Unit]
Description=macrokey

[Service]
ExecStart=${DIR}/macrokey
Restart=on-failure
#StartLimitIntervalSec=60
#StartLimitBurst=4

[Install]
WantedBy=default.target
EOL

    systemctl --user enable macrokey.service --now
    systemctl --user status macrokey.service
}


function fn_root_service {
    DIR="$( cd "$( dirname "$0" )" && pwd )"

sudo tee /etc/systemd/system//macrokey.service > /dev/null << EOL 
[Unit]
Description=macrokey

[Service]
ExecStart=${DIR}/macrokey
Restart=on-failure

[Install]
WantedBy=default.target
EOL

    sudo systemctl enable macrokey.service --now
    sudo systemctl status macrokey.service
}


function fn_run {
    cp ./target/release/macrokey macrokey
    ./macrokey
}


function fn_build {
    cargo build --release
    cp ./target/release/macrokey macrokey
}


function fn_build_debug {
    cargo build
}


function fn_run_debug {
    ./target/debug/macrokey
}


# pass all args
main "$@"
