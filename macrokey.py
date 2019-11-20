#!/usr/bin/env python

# Then this script will be able to be executed:
#   # sudo ./macrokey.py
#
# Can also accept a class to instantiate which allows for custom macros for custom games, eg:
#   # sudo ./macrokey.py TheForest
#

from subprocess import call
import macrokey
import threading
import sys

# https://github.com/spotify/linux/blob/master/include/linux/input.h
EV_KEY = 1

EV_PRESSED = 1
EV_RELEASED = 0
EV_REPEAT = 2

BTN_LEFT = 0x110
BTN_RIGHT = 0x111
BTN_MIDDLE = 0x112

KEY_A = 30
KEY_B = 48
KEY_C = 46

# Repeatedly click the left mouse
class ClickRepeatTimer(threading.Thread):
    key = BTN_LEFT
    pressed_time = 0.1
    released_time = 3.5
    
    def __init__(self, p_key, p_pressed_time, p_released_time):
        threading.Thread.__init__(self)
        self.event = threading.Event()
        self.key = p_key
        self.pressed_time = p_pressed_time
        self.released_time = p_released_time

    def run(self):
        while not self.event.is_set():
            print("Click")
            macrokey.send_event_to_virtual_device(self.key, EV_PRESSED)
            self.event.wait(self.pressed_time)
            macrokey.send_event_to_virtual_device(self.key, EV_RELEASED)
            self.event.wait(self.released_time)

    def stop(self):
        self.event.set()
        # ensure released at end!
        macrokey.send_event_to_virtual_device(self.key, EV_PRESSED)
        macrokey.send_event_to_virtual_device(self.key, EV_RELEASED) 
        
# ------------------------------------------------------------------------------------------

#
# Left pedal - repeatedly hit left click
# Middle pedal - hold left click
# Righ tpedal - repeatedly his right click
#
class Default:
    leftClickRepeatTimer = None
    rightClickRepeatTimer = None

    def process_input(self, p_type, p_code, p_value): 
        # emulate repeating left mouse click
        if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_A):
            if (self.leftClickRepeatTimer is None):
                print("Starting left click repeat timer")
                self.leftClickRepeatTimer = ClickRepeatTimer(BTN_LEFT, 0.1, 0.1)
                self.leftClickRepeatTimer.start()

        if (p_type == EV_KEY and p_value == EV_RELEASED and p_code == KEY_A):
            if (self.leftClickRepeatTimer):
                self.leftClickRepeatTimer.stop()
                self.leftClickRepeatTimer = None
                print("Stopping left click repeat timer")


        # emulate repeating right mouse click
        if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_C):
            if (self.rightClickRepeatTimer is None):
                print("Starting right click repeat timer")
                self.rightClickRepeatTimer = ClickRepeatTimer(BTN_RIGHT, 0.1, 0.1)
                self.rightClickRepeatTimer.start()

        if (p_type == EV_KEY and p_value == EV_RELEASED and p_code == KEY_C):
            if (self.rightClickRepeatTimer):
                self.rightClickRepeatTimer.stop()
                self.rightClickRepeatTimer = None
                print("Stopping right click repeat timer")


        # emulate holding left mouse hold
        if (p_type == EV_KEY):
            if (p_code == KEY_B):
                macrokey.send_event_to_virtual_device(BTN_LEFT, p_value)
        
# ------------------------------------------------------------------------------------------


def debug(p_type, p_code, p_value):
    global debug_enabled, last_debug
    if not debug_enabled:
        return
    
    new_debug = "type: " + str(p_type) + " code:" + str(p_code) + " value: " + str(p_value)
    if last_debug != new_debug:
        print("Debug: " + new_debug)
        last_debug = new_debug
    
    return


def process_input(p_type, p_code, p_value):
    debug(p_type, p_code, p_value)
    callbackInst.process_input(p_type, p_code, p_value)
    return


def main():
    debug_enabled = False
    className = "Default"
    last_debug = ''
        
    if (len(sys.argv) > 1):
        className = sys.argv[1]

    if (not hasattr(sys.modules[__name__], className)):
        print("Class not found: " + className)
        sys.exit(1)
    else:
        print("Running with class: " + className)

    callbackClass = getattr(sys.modules[__name__], className)    
    callbackInst = callbackClass()

    macrokey.set_py_callback(process_input)
    macrokey.add_device("keyboard")
    macrokey.run()


if __name__ == '__main__':
    main()
