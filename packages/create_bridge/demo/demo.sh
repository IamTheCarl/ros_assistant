#!/usr/bin/env bash

ros2 topic pub --once /create_bridge/drive/straight std_msgs/msg/Int16 'data: -100'

sleep 1s

# ros2 topic pub --once /drive/straight std_msgs/msg/Int16 'data: 0'
ros2 topic pub --once /create_bridge/drive/left std_msgs/msg/Int16 'data: 188'

sleep 2s

ros2 topic pub --once /create_bridge/drive/straight std_msgs/msg/Int16 'data: 0'
ros2 topic pub --once /create_bridge/display_text std_msgs/msg/String 'data: "make"'

sleep 2s

ros2 topic pub --once /create_bridge/display_text std_msgs/msg/String 'data: ""'
ros2 topic pub --once /create_bridge/drive/right std_msgs/msg/Int16 'data: 188'

sleep 2s

ros2 topic pub --once /create_bridge/dock std_msgs/msg/Empty
