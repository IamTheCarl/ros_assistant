# SSH Jumping

ROS allows nodes to communicate between multiple devices. A very typical setup for robots that use multiple devices is to have an internal network that cannot be reached from the outside.

ROS Assistant supports ssh jumping. This enables you to ssh into a public facing host that will then automatically forward the ssh connection to hosts on the internal network.

This example sets up such a configuration, to an extreme. You can jump to a host on the internal network, and then to a host within yet another sub-network. To visualize:

```
[you] --ssh--> [host a] --ssh--> [host b] --ssh--> [host c]
```

# Deploying

You'll need to initialize each host by hand. The first two are x86 machines that you can just bringup with an unattended install thumb drive. The third one is a Raspberry Pi (you can actually use whatever you want as long as you adjust the host files as needed. I just didn't have 3 x86 machines available so the last one is a Pi I happen to have)

Once the machines have been initially brought up, you can push updates to them over ssh through the whole chain.

# Building installer ISOs

If you choose to build installer ISOs for this, you'll find that system-c won't build since it's meant for a Raspberry Pi. You can ignore it using the host select feature of  the deployment system.

```
rass deploy --hosts 'system-[ab]' installer
```