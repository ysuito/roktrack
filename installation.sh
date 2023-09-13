#!/bin/sh

# This is just a little script that can be downloaded from the internet to
# install roktrack. 

set -u

mkdir roktrack
cd roktrack
curl -OL https://github.com/ysuito/roktrack/releases/download/v0.1.0-alpha/roktrack-v0.1.0-alpha.zip
unzip roktrack-v0.1.0-alpha.zip
sudo cp libonnxruntime.so.1.15.1 /usr/lib/
rm libonnxruntime.so.1.15.1
rm roktrack-v0.1.0-alpha.zip
