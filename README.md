# Roktrack Pylon-Guided Robotic Mower
<div>
  <img src="asset/img/swarm_horizonal.JPG">
</div>

Open source robotic mower using image recognition technology. No GPS. No boundary wire.

- Just place the pylon and flip the switch.
- Solar charging and AC charging.
- As a surveillance camera that can detect person and animals while charging.

> [!WARNING]  
> Fast-spinning  blades are very dangerous. If you make this machine by yourself, please be very careful and ensure the safety of your surroundings before using it.

## Demo
### Single Operation
<div>
  <img src="asset/img/one_node_mowing.gif" height="400" width="600">
</div>

### Parallel Operation
<div>
  <img src="asset/img/four_node_mowing.gif" height="400" width="600">
</div>

# Requirement
* Raspberry Pi 3A+ or Raspberry Pi 4B or Raspberry Pi Zero 2W(without speaking)
* libv4l-dev
* libssl-dev

# Installation
```bash
sudo apt install -y libv4l-dev libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ysuito/roktrack/master/installation.sh | sh
```

# Usage
Surround the area to be mowed with pylons (traffic cones), place Roktrack and execute the following command.
```bash
cd roktrack
sudo ./roktrack
```

# Auto-Startup
If you want to start it at the same time as system startup, create a file with the following contents as /lib/systemd/system/roktrack.service.

```/lib/systemd/system/roktrack.service
[Unit]
Description = roktrack

[Service]
ExecStart=/home/pi/roktrack/roktrack
Restart=always
Type=simple
WorkingDirectory=/home/pi/roktrack

[Install]
WantedBy=multi-user.target
```

Load it and activate it.
```bash
sudo systemctl daemon-reload
sudo systemctl enable roktrack.service
```

Finally, set the Raspberry Pi to Read Only.
```bash
sudo raspi-config nonint enable_overlayfs
sudo systemctl reboot
```

> [!Note]  
> If you are using a Raspberry Pi3A+ or Zero 2W, please refer to the [hackaday log](https://hackaday.io/project/190977-roktrack-pylon-guided-mower/log/220570-expand-the-available-memory-area-of-rpi3a-made-read-only) to reserve memory.

# License
The source code is licensed GPL v3.0. The files under the assets and hardware directories are licensed CC BY-NC-SA 4.0,see LICENSE.

## Links
### English
- [hackaday.io](https://hackaday.io/project/190977-roktrack-pylon-guided-mower)
### Japanese
- [Roktrack1 protopedia](https://protopedia.net/prototype/3357)
- [Roktrack2 protopedia](https://protopedia.net/prototype/3788)

