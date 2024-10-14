#!/usr/bin/env bash

# We have to disable the firewall on the robot first.
# rass firewall disable 
# 
# # For some reason I have to restart the nodes on the robot to see them from here.
# ssh root@rosetta.lan 'systemctl restart teleop-twist-joy.service'
# 
# sleep 5
# 
# # Now we can bring up a joystick node to control the robot.
# ros2 node list
ros2 run joy joy_node 
# --ros-args -r __node:=remote_joy