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
    1) Run
    2) Build
    3) Build Debug
    4) Run Debug
    5) Install
    
    *) Any key to exit
    :" ans;
    reset
    case $ans in
        1) fn_run ;;
        2) fn_build ;;
        3) fn_build_debug ;;
        4) fn_run_debug ;;
        5) fn_install ;;
        *) $SHELL ;;
    esac
    done
}


function fn_run {
    cp ./build/libmacrokey.so macrokey.so
    python macrokey.py
}


function fn_build {
    mkdir -p build
    cd build
    cmake .. -DCMAKE_BUILD_TYPE=Release
    make
    cd ..
}


function fn_build_debug {
    mkdir -p build
    cd build
    cmake .. -DCMAKE_BUILD_TYPE=Debug
    make
    cd ..
}


function fn_run_debug {
    #sudo gdb {} -'.format(config['path']['debug']))
    echo "TODO: fix"
}


function fn_install {
    yay -S --noconfirm --needed boost gdb
}


# pass all args
main "$@"
