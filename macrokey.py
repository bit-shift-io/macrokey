#!/usr/bin/env python

# To run this script:
# ./macrokey.py
#
# Can also accept a class to instantiate which allows for custom macros for custom games, eg:
# ./macrokey.py TheForest

from subprocess import call
import macrokey
import sys
from default import Default
from remote import Remote

# globals
debug_enabled = False
callback_instance = ""
last_debug = ''

def log(str=''):
    print(str)
    return


def debug(p_type, p_code, p_value, p_device):
    global debug_enabled, last_debug
    if not debug_enabled:
        return
    
    new_debug = "type: " + str(p_type) + " code:" + str(p_code) + " value: " + str(p_value) + "dev: " + str(p_device)
    if last_debug != new_debug:
        log("Debug: " + new_debug)
        last_debug = new_debug
    
    return


def process_input(p_type, p_code, p_value, p_device):
    global callback_instance
    debug(p_type, p_code, p_value, p_device)
    callback_instance.process_input(p_type, p_code, p_value, p_device)
    return


def main():
    # get class from args
    global callback_instance

    class_name = "Default"
    if (len(sys.argv) > 1):
        class_name = sys.argv[1]

    if (not hasattr(sys.modules[__name__], class_name)):
        log("Class not found: " + class_name)
        sys.exit(1)
    else:
        log("Profile: " + class_name)

    callback_class = getattr(sys.modules[__name__], class_name)    
    callback_instance = callback_class()
    macrokey.set_py_callback(process_input)

    # init
    result = callback_instance.init()
    if (not result):
        log("Init: device not found")
        sys.exit(1)

    # run
    try:
        # start c++ macrokey
        macrokey.run()
    except KeyboardInterrupt:
        macrokey.done()

    return


if __name__ == '__main__':
    main()