#!/usr/bin/env bash

# We have to disable the firewall on the robot first.
rass firewall pierce 

trap "exit" INT TERM
trap "kill 0" EXIT

ros2 run joy joy_node --ros-args \
    -r __node:=remote_joy \
    -r /joy:=/remote_joy &
ros2 run teleop_twist_joy teleop_node --ros-args \
    -r __node:=remote_teleop \
    -r /joy:=/remote_joy \
    -p enable_button:=4 \
    -p enable_turbo_button:=5 \
    -p axis_linear.x:=1 \
    -p axis_angular.yaw:=0 \
    -p scale_angular.yaw:=1.0 &

rviz2 -d ./rviz_layout.rviz