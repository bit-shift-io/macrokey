#!/usr/bin/env python
#-*- coding: utf-8 -*-

import os
import sys
import subprocess
import inspect
import multiprocessing
import shutil
import getpass

config = {
    'display_name': 'macrokey',
    'logToFile': False,
    'path': {
        'cwd': os.path.abspath(os.getcwd()),
        'release': os.path.abspath('./build/libmacrokey.so'),
        'debug': os.path.abspath('./build/macrokey'),
        'build': os.path.abspath('./build/'),
    },
}


def main():
    menu = {}
    menu['1'] = ['Build', 'build']
    menu['2'] = ['Build debug', 'build_debug']
    menu['3'] = ['Debug (GDB)', 'debug']
    menu['4'] = ['Start', 'start']
    menu['r'] = ['Requirements', 'requirements']

    print('\n********************')
    print ('    {}'.format(config['display_name']))
    print('********************')
    for item in menu:
        print (' ' + item + '. ' + menu[item][0])
        
    selection = input('> ')
    # check if in menu
    if selection in menu:
        eval(menu[selection][1] + '()')

    # exec function
    if '()' in selection:
        eval(selection)

    main()
    return


def start():
    run('cp ./build/libmacrokey.so macrokey.so')
    run_sudo('python macrokey.py')
    return


def build_debug():
    run('''
    mkdir -p build
    cd build
    cmake .. -DCMAKE_BUILD_TYPE=Debug
    make
    cd ..
    ''')
    return
    

def debug():
    run_sudo('gdb {} -'.format(config['path']['debug']))
    return


def build():
    run('''
    mkdir -p build
    cd build
    cmake .. -DCMAKE_BUILD_TYPE=Release
    make
    cd ..
    ''')
    return
    

def requirements():
    run('''
    yay -S --noconfirm --needed boost gdb
    ''')
    return


def log(str=''):
    print(str)
    if not config['logToFile']:
        return

    with open("log.txt", "a") as f:
        f.write(str + '\n')
    return
    

# run commands
# params:
# cwd
# show cmd
def run(command, params = {}):
    # clean command
    cmd = inspect.cleandoc(command)
    
    # show output
    show_cmd = False
    if 'show_cmd' in params:
        show_cmd = params['show_cmd']

    if show_cmd:
        print(cmd + '\n')
        
    working_dir = os.getcwd()
    if 'cwd' in params:
        working_dir = params['cwd']
        
    # exec
    subprocess.run(cmd, shell=True, cwd=working_dir)
    return


def run_sudo(command, params = {}):
    cmd = inspect.cleandoc(command)
    password = getpass.getpass('[sudo] password: ')
    cmd = 'echo {}|sudo -S {}'.format(password, cmd)
    run(cmd, params)
    return


if __name__ == '__main__':
    os.system('cls||clear')
    main()
