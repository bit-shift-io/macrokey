# -*- coding: utf_8 -*-

from threading import Timer,Thread,Event

class ThreadTimer():

   def __init__(self,t,hFunction):
      self.t=t
      self.hFunction = hFunction
      self.thread = Timer(0,self.handle_function)
      self.thread.setDaemon(True)

   def handle_function(self):
      self.hFunction()
      self.thread = Timer(self.t,self.handle_function)
      self.thread.start()

   def start(self):
      self.thread.start()

   def cancel(self):
      self.thread.cancel()
