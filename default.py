#!/usr/bin/env python

from subprocess import call
import threading
import macrokey

# globals
right_click_hold = True # if false, right click taps the button, if true, right click is held

# device list
dev_foot = -1
dev_keyboard = -1

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


# ------------------------------------------------------------------------------------------
# Repeatedly click
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
    ctrl_down = False
    alt_down = False
    capslock_down = False # may be a way to read from system?
    modifier_down = False
    key_repeat_map = {}

    def process_foot(self, p_type, p_code, p_value, p_device, p_name):
        # emulate repeating left mouse click
        if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_A):
            if (self.leftClickRepeatTimer is None):
                self.leftClickRepeatTimer = ClickRepeatTimer(BTN_LEFT, 0.1, 0.1)
                self.leftClickRepeatTimer.start()

        if (p_type == EV_KEY and p_value == EV_RELEASED and p_code == KEY_A):
            if (self.leftClickRepeatTimer):
                self.leftClickRepeatTimer.stop()
                self.leftClickRepeatTimer = None


        # emulate repeating right mouse click
        if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_C):
            if (right_click_hold):
                macrokey.send_event_to_virtual_device(BTN_RIGHT, EV_PRESSED)
            elif (self.rightClickRepeatTimer is None):
                self.rightClickRepeatTimer = ClickRepeatTimer(BTN_RIGHT, 0.1, 0.1)
                self.rightClickRepeatTimer.start()

        if (p_type == EV_KEY and p_value == EV_RELEASED and p_code == KEY_C):
            if (right_click_hold):
                macrokey.send_event_to_virtual_device(BTN_RIGHT, EV_RELEASED)
            elif (self.rightClickRepeatTimer):
                self.rightClickRepeatTimer.stop()
                self.rightClickRepeatTimer = None


        # emulate holding left mouse hold
        if (p_type == EV_KEY):
            if (p_code == KEY_B):
                macrokey.send_event_to_virtual_device(BTN_LEFT, p_value)
        
        return


    def process_keyboard(self, p_type, p_code, p_value, p_device, p_name):
        # modifier keys
        if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_CTRL):
            self.ctrl_down = True
        if (p_type == EV_KEY and p_value == EV_PRESSED and p_code == KEY_ALT):
            self.alt_down = True
        if (p_type == EV_KEY and p_value == EV_RELEASED and p_code == KEY_CTRL):
            self.ctrl_down = False
        if (p_type == EV_KEY and p_value == EV_RELEASED and p_code == KEY_ALT):
            self.alt_down = False

        # modifier key boolean
        if (self.ctrl_down or self.alt_down):
            self.modifier_down = True
        else:
            self.modifier_down = False

        # timers start
        # with key combo ctrl + alt + key
        if (self.ctrl_down and self.alt_down and p_code != KEY_ALT and p_code != KEY_CTRL):
            if (p_type == EV_KEY and p_value == EV_PRESSED):
                if (p_code not in self.key_repeat_map):
                    self.key_repeat_map[p_code] = ClickRepeatTimer(p_code, 0.1, 0.1)
                    self.key_repeat_map[p_code].start()

        # timers end
        # with single key press
        # ensure no modifier keys are active
        if (p_type == EV_KEY and p_value == EV_PRESSED and self.modifier_down == False):
            if (p_code in self.key_repeat_map):
                self.key_repeat_map[p_code].stop()
                del self.key_repeat_map[p_code]

            # delete all timers
            # tilde key
            if (p_code == KEY_TILDE):
                for key in self.key_repeat_map:
                    self.key_repeat_map[key].stop()
                self.key_repeat_map = {}

            # toggle all timers on/off
            # caps lock
            if (p_code == KEY_CAPSLOCK):
                self.capslock_down = not self.capslock_down

                if (self.capslock_down):
                    for key in self.key_repeat_map:
                        self.key_repeat_map[key].stop()
                else:
                    for key in self.key_repeat_map:
                        # need to recreate the timers first
                        self.key_repeat_map[key] = ClickRepeatTimer(key, 0.1, 0.1)
                        self.key_repeat_map[key].start()

        return


    def process_input(self, p_type, p_code, p_value, p_device, p_name):
        if (p_device == dev_foot):
            self.process_foot(p_type, p_code, p_value, p_device, p_name)
        if (p_device == dev_keyboard):
            self.process_keyboard(p_type, p_code, p_value, p_device, p_name)   
        return
    

    def init(self):
        global dev_foot
        global dev_keyboard

        # add devices
        dev_foot = macrokey.open_device("FootSwitch3-F1.8 Keyboard", True)
        if (dev_foot == -1):
            dev_foot = macrokey.open_device("HID 413d:2107 Keyboard", True)

        # need some regex eventually
        dev_keyboard = macrokey.open_device("SONiX USB DEVICE", False)
        if (dev_keyboard == -1):
            dev_keyboard = macrokey.open_device("/dev/input/event31", False)
        if (dev_keyboard == -1):
            dev_keyboard = macrokey.open_device("HOLTEK USB-HID Keyboard", False)
        if (dev_keyboard == -1):
            dev_keyboard = macrokey.open_device("keyboard", False)
        if (dev_keyboard == -1):
            dev_keyboard = macrokey.open_device("Keyboard", False)

        if dev_keyboard == -1:
            return False

        return True
            