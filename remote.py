#!/usr/bin/env python

from subprocess import call
import threading
import macrokey


# device list
dev_remote = -1

# https://github.com/spotify/linux/blob/master/include/linux/input.h
EV_KEY = 1

EV_PRESSED = 1
EV_RELEASED = 0
EV_REPEAT = 2

BTN_LEFT = 0x110
BTN_RIGHT = 0x111
BTN_MIDDLE = 0x112

# mapping for foot pedals
KEY_A = 30
KEY_B = 48
KEY_C = 46

KEY_CTRL = 29
KEY_ALT = 56
KEY_SHIFT = 42
KEY_TILDE = 41
KEY_CAPSLOCK = 58
KEY_TAB = 15
KEY_META = 125 # win key
KEY_ESC = 1


class Remote:
    leftClickRepeatTimer = None
    rightClickRepeatTimer = None
    ctrl_down = False
    alt_down = False
    capslock_down = False # may be a way to read from system?
    modifier_down = False
    key_repeat_map = {}

    def process_input(self, p_type, p_code, p_value, p_device):
        # emulate holding left mouse hold
        if (p_type == EV_KEY):
            if (p_code == KEY_B):
                macrokey.send_event_to_virtual_device(BTN_LEFT, p_value)
        
        return


    def process_input(self, p_type, p_code, p_value, p_device):
        if (p_device != dev_remote):
            return

        macrokey.log("process remote")

        return
    

    def init(self):
        global dev_remote

        # add devices
        dev_remote = macrokey.open_device("FootSwitch3-F1.8 Keyboard", True)
        if (dev_remote == -1):
            return False

        return True