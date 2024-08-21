# macrokey
A simple C++ & Python alternative to auto hotkey (AHK) for linux. 

## Build/Install/Run
Run ```./build.sh```

## Run
Run ```./build.sh```
or  
```./macrokey.py {GameName}```

Where {GameName} currently can be:  
Default - use a default profile  
TheForest - specialised left clicking to support cutting down trees in The Forest  

## Default
Ctrl + Alt + Key - starts a repeating timer for that key, press the key again without the modifiers and it will delete the timer.  
~ (Tilde) - deletes all repeat timers.  
Caps Lock - toggles all repeat timers on/off.  