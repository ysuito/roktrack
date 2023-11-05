# Installation
Use Raspberry Pi Imager to write "Raspberry Pi OS(64-bit) Lite" to the SD card. Also, set up a wifi connection and connect to the Internet.
Insert the SD card into the raspberry pi and perform the following steps in a place where wifi is connected.

## OS Setup
Add the following to the `/boot/config.txt`.

```text:/boot/config.txt
# Disable the PWR LED
dtparam=pwr_led_trigger=timer
dtparam=pwr_led_activelow=off

# Disable the Activity LED
dtparam=act_led_trigger=none
dtparam=act_led_activelow=off

# GPIO PWM Audio
dtoverlay=audremap,pins_12_13

# Reduce GPU Mem
gpu_mem=16
```

If you are using RPi3A+ or Zero 2, apply the following memory saving settings.

Rewrite `/usr/bin/raspi-config` as follows. (UPDATE will rewrite the default configuration!)
```bash:/usr/bin/raspi-config
- mount -t tmpfs tmpfs /upper
+ mount -t tmpfs -o size=50M tmpfs /upper
```

```bash:
# expand swap
sudo sed -i -e "s/^CONF_SWAPSIZE=.*/CONF_SWAPSIZE=0/g" /etc/dphys-swapfile # disable swap
# disable services
sudo systemctl disable rsyslog.service
sudo systemctl disable polkit.service
sudo systemctl disable ModemManager.service
sudo systemctl disable bluetooth
```


# Install App
```bash
sudo apt install -y libv4l-dev libssl-dev mpg123 bluez-hcidump
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ysuito/roktrack/master/installation.sh | sh
```

# Auto-Startup
If you want to start it at the same time as system startup, create a file with the following contents as /lib/systemd/system/roktrack.service.

```/lib/systemd/system/roktrack.service
# /lib/systemd/system/roktrack.service
[Unit]
Description = roktrack
After=network-online.target
Wants=network-online.target

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

Maximize the volume
```
alsamixer # Turn volume to maximum
```

Finally, set the Raspberry Pi to Read Only.
```bash
sudo raspi-config nonint enable_overlayfs
sudo systemctl reboot
```
