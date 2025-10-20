#!/bin/bash

Xvfb :99 -screen 0 1280x1024x24 -nolisten tcp &
export DISPLAY=:99

/apps/huly-cef-manager --cache-dir /cefcache --cef-exe /apps/huly-cef-websockets $@