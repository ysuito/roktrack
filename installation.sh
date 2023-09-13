#!/bin/sh

# This is just a little script that can be downloaded from the internet to
# install roktrack. 

set -u

sudo apt install libv4l-dev libssl-dev
git clone https://github.com/ysuito/roktrack.git
cd roktrack
# curl -O https://github.com/ysuito/roktrack/libonnxruntime.so.1.15.1
# sudo cp libonnxruntime.so.1.15.1 /usr/lib/
# rm libonnxruntime.so.1.15.1
# curl -O https://github.com/ysuito/roktrack/roktrack