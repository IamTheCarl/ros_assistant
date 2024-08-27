# NVidia Xavier AGX

I have an NVidia Xavier AGX devkit. It was a mistake. I do not recommend it. The official tools for it are very obtuse and critical documentation difficult to obtain. All of that asside, I happen to have one and wanted to prove it possible for this system to work with the device.

## Setup

You need to install UEFI compatible firmware to your Xavier. [Start here](https://github.com/anduril/jetpack-nixos) to do that. Once you have the UEFI firmware installed, you're done there. You don't need to install NixOS. Once you have the firmware setup, you can generate the disk image from this project and write it to an SD card. Boot off the SD Card.

By the way, this will take *hours* to build, so sit tight and be patient.

## But I want to boot off the device's internal storage

That's cool. I haven't figured out how to do that. I might do it later if I find the motivation (unlikely, I have sunk more time on this board than it's worth)

I will hazard a guess as to what it'll take though. Something that really makes the Xavier a real special child is that it has a bunch of special partitions with a really specific layout. Mess with those partitions and you'll break the firmware and be left with a brick. You'll have to re-flash the board with the UEFI firmware (mentioned in the setup section above) to fix your board.

So you'll need to boot the board off a USB thumb drive, SD card, or some other medium that isn't the eMMC. From there, manually install NixOS and the UEFI bootloader. I uhh... don't remember which partitions they need to go into but I am aware that they should already exist. If you can get a Nix system running, from that point you can configure it up with an ssh server. If ssh is working, then at that point you can deploy to the board over ssh. 