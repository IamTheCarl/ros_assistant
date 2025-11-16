# Rosetta Robot

Rosetta is a Roomba 600 with a Raspberry Pi attached to the top.

![image](doc/rosetta_docked.jpg)

The purpose of this robot is to prove the feasibility of RASS by building a simple robot and developing software for it.

# Features

Currently Rosetta can be connected to a Bluetooth game pad (a Playstation 4 controller) and be driven around. Wifi and Bluetooth have to be configured manually over SSH. Ethernet is pre-configured in the initially generated image. Make sure to replace my public keys with your own so you can log in with ssh.

# Namesake

Rosetta is named after the Rosetta Stone. Rosetta starts with ROS, but was also a stone with three languages. Rosetta Robot contains ROS nods written in C++, Python, and Rust.