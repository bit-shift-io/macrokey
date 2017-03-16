#!/usr/bin/env python3

# Requirements:
# windows:
# PyHook - pip3 install pyhook
# PyPiWin32 - pip3 install pypiwin32
# PyUserInput - pip3 install pyuserinput

# linux:
# python3-xlib & python3-pip - sudo apt-get install python3-xlib python3-pip -y
# PyUserInput - pip3 install pyuserinput


__author__ = 'Bronson Mathews'

# imports
import time
import sys
import thread_timer
from pymouse import PyMouse
from pykeyboard import PyKeyboard
from pymouse import PyMouseEvent
from pykeyboard import PyKeyboardEvent

# emulate our mouse and keyboard
Mouse = PyMouse()
Keyboard = PyKeyboard() 
	
# dictionary
keyState = {}
mouseState = {}

# class for inputs
class MouseEventListener(PyMouseEvent):
	def click(self, x, y, button, press):
		#print ('mouse click')
		global mouseState
		mouseState[button] = press
		OnInputEvent('mouse', button, press)

class KeyboardEventListener(PyKeyboardEvent):
	def tap(self, keycode, character, press):  # press is boolean; True for press, False for release
		global keyState
		keyState[keycode] = press
		OnInputEvent('keyboard', keycode, press)
		


# toggles
debug = True

# globals
IgnoreEvent = False
Exit = False

global M
global K
M = MouseEventListener()
K = KeyboardEventListener()

# define keys
pressW = False
pressLMB = False
tapLMB = False



# Mouse + Keyboard Input Events	
def OnInputEvent(device, index, press):
	if IgnoreEvent:
		return

	if debug:
		print (device, index, press)
	
	LShift = keyState[50] if (50 in keyState) else False
	RShift = keyState[62] if (62 in keyState) else False
	LCtrl = keyState[37] if (37 in keyState) else False
	RCtrl = keyState[105] if (105 in keyState) else False
	LAlt = keyState[64] if (64 in keyState) else False
	RAlt = keyState[108] if (108 in keyState) else False

	# keyboard shortcuts
	if device == 'keyboard':
		
		# Ctrl + Alt + X = Exit
		if LCtrl and LAlt and index == 53 and press:
			global Exit
			Exit = True
			
		# Alt + W = hold W
		global pressW
		
		if LAlt and index == 25 and press:
			pressW = not pressW
			return
			
		if pressW and index == 25 and press:
			pressW = False		
	
	# mouse shortcuts
	if device == 'mouse':
		
		# Alt + LMB = hold LMB
		global pressLMB
		
		if LAlt and index == 1 and press:
			pressLMB = not pressLMB
			return
			
		if pressLMB and index == 1 and press:
			pressLMB = False
			
		# Ctrl + LMB = click LMB
		global tapLMB
		
		if LCtrl and index == 1 and press:
			tapLMB = not tapLMB
			return
			
		if tapLMB and index == 1 and press:
			tapLMB = False	
	
	return

	
# Hide Console
def hide():
	import win32console,win32gui
	window = win32console.GetConsoleWindow()
	win32gui.ShowWindow(window,0)
	return       
	
# Update
def update():

	IgnoreEvent = True
	
	if pressW:
		Keyboard.press_key('W')
		
	if pressLMB:
		pos = Mouse.position()
		Mouse.press(pos[0],pos[1],1)
		
	if tapLMB:
		pos = Mouse.position()
		print ('tap')
		Mouse.press(pos[0],pos[1],1)
		Mouse.release(pos[0],pos[1],1)

	IgnoreEvent = False
	return   

	
# Main Loop
if __name__ == '__main__':
	# Hide Console
	if debug == False:
		hide()
	
	# Listeners
	M.start()
	K.start()
	
	# start update loop in a thread
	updateThread = thread_timer.ThreadTimer(0.1, update)
	updateThread.start()
	
	# Infinite Loop
	while not Exit:
		pass
		
	sys.exit()
