#!/bin/bash

qmake -project
qmake -makefile
make
sudo ./macrokey
