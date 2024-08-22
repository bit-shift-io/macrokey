#!/usr/bin/env python

import os
from subprocess import call
import subprocess
import threading
import macrokey

# device list
dev_system = -1
dev_consumer = -1

# types
# https://github.com/spotify/linux/blob/master/include/linux/input.h
EV_KEY = 1

# values
EV_PRESSED = 1
EV_RELEASED = 0
EV_REPEAT = 2

# key codes for system
KEY_POWER = 116

# key codes for consumer
KEY_HOME = 172
KEY_VOLUME_UP = 115
KEY_VOLUME_DOWN = 114
KEY_PLAY_PAUSE = 164
KEY_REWIND = 165
KEY_FASTFORWARD = 163
KEY_START = 168
KEY_END = 208
KEY_ZOOM_IN = 418
KEY_ZOOM_OUT = 419
KEY_MEDIA = 171
KEY_HELP = 155
KEY_MUTE = 113


def log(str=''):
    print(str)
    return


def script_path(): 
    dir = os.path.dirname(os.path.realpath(__file__))
    log(dir)
    return


def run_command(command):
    result = ""
    try:
        result = subprocess.check_output(command, shell=True).decode("utf-8")
    except subprocess.CalledProcessError as e:
        log(e.output.decode("utf-8"))
    return result



def send_cec_command(state):
    # commands will fail if cec device not found
    if (state):
        log("turning screen on")
        run_command("echo 'on 0' | cec-client -s")
    else:
        log("turning screen off")
        run_command("echo 'standby 0' | cec-client -s")


def toggle_display():
    # get screen on/off state
    state = run_command("echo 'pow 0' | cec-client -s -d 1")
    if ('power status: on' in state):
        send_cec_command(False)
    else:
        send_cec_command(True)
    return


class Remote:
    def process_consumer(self, p_type, p_code, p_value, p_device, p_name):
        # log("process consumer remote")

        # if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_HOME):
        #     log("home")
        
        return
    
    
    def process_system(self, p_type, p_code, p_value, p_device, p_name):
        if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_POWER):
            toggle_display()
        return


    def process_input(self, p_type, p_code, p_value, p_device, p_name):
        if p_device == dev_consumer:
            self.process_consumer(p_type, p_code, p_value, p_device, p_name)
        elif p_device == dev_system:
            self.process_system(p_type, p_code, p_value, p_device, p_name)
        return
    

    def init(self):
        global dev_consumer, dev_system

        # add devices
        #device_list.append(macrokey.open_device("Usb Audio Device Mouse", True))
        #dev_consumer = macrokey.open_device("Usb Audio Device Consumer Control", True)
        dev_system = macrokey.open_device("Usb Audio Device System Control", True)
        #device_list.append(macrokey.open_device("Usb Audio Device", True))

        if (dev_system == -1):
            return False

        return True