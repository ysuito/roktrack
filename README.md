# Roktrack
Roktrack is a pylon-guided robotic mower.

<div>
  <img src="asset/img/one_node_mowing.gif" height="400" width="600">
</div>

# Requirement
* libv4l-dev
* Raspberry Pi 3A+ or 4B
* Rust

# Installation
```bash
sudo apt install libv4l-dev
git clone https://github.com/ysuito/roktrack.git
cd roktrack
cargo b -r
sudo cp target/release/libonnxruntime.so.1.15.1 /usr/lib/
cp target/release/roktrack ./
```

# Usage
Surround the area to be mowed with pylons (traffic cones), place Roktrack and execute the following command.
```bash
sudo roktrack
```

# License
The source code is licensed GPL v3.0. The files under the assets directory are licensed CC BY-NC-SA 4.0,see LICENSE.

## Links
- https://hackaday.io/project/190977-roktrack-pylon-guided-mower
- https://protopedia.net/prototype/3357
- https://protopedia.net/prototype/3788

