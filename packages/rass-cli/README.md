# RASS-cli

A command line tool to build and deploy software to robots. This is done using the Nix package manager and ssh.

A strong focus is on easy reproducibility of build artifacts and idempotent deployment. The previous state of your robot will not influence the new state after deployment. No more will you be forgetting you didn't install a package, or that your coworker left an experimental and broken node on there. No more will you rebase your branch and discover that a new upstream dependency is required and you have to spend the next 20 minutes figuring out what it is and how to install it. When you commit code to your repository, that commit will always produce the exact same software stack on the robot.

## Examples

Examples can be found under the [in the example projects](example_projects).

## Deployment methods

### Disk image

Create a disk image for your robot. Write the image using the `dd` command or another image writing tool of your choice. The machine will boot into the OS generated by RASS.

Run `rass deploy disk` from the root of any of the following example projects:
* [Raspberry Pi](example_projects/raspberry_pi)
* [NVidia Xavier](example_projects/nvidia-xavier-agx)
* [Generic x86](example_projects/nvidia-xavier-agx)
* [SSH Jumping](example_projects/ssh_jumping)
  * This will produce multiple disk images, one for each host within the robot.

### Installer

It is common for hard drives to be glued into the robot. Removing this drive to install an OS is impractical. You can create an installer ISO that will perform an unattended installation and automatically turn off the target computer upon completion. Typically you will write this ISO to a USB thumb drive, although any medium you can boot from should be capable of this.

Run `rass deploy installer` from the root of any of the following example projects:
* [Generic x86](example_projects/nvidia-xavier-agx)
* [SSH Jumping](example_projects/ssh_jumping)
  * This will produce multiple installer images, one for each host within the robot.

### SSH

If your robot is already running and you can ssh into it, you can push updates to it. This is much faster and convenient than disk images or installers. These updates are typically deployed in test mode as well, meaning that rebooting the robot will revert the changes, mitigating risks of a bad deployment. (use the `--switch` flag to make an update persist between reboots)

* [Raspberry Pi](example_projects/raspberry_pi)
* [NVidia Xavier](example_projects/nvidia-xavier-agx)
* [Generic x86](example_projects/nvidia-xavier-agx)
* [SSH Jumping](example_projects/ssh_jumping)
  * This example expects computers to be hidden behind other computers and will ssh tunnel through other computers to reach ones deeper in the robot.

# Installing

RASS depends on Nix to function, and therefore is distributed using Nix. You must [install nix](https://nixos.org/download/) before you can install RASS. After you have installed Nix, you can create a nix shell with it by cloning this repository and running `nix-shell path/to/shell.nix`, the shell.nix being [this one](test-drive-shell.nix). More traditional methods of installing nix packages are not yet available as this repository is not yet available as a nix channel.

## Cross Compilation

While most packages can be cross compiled without any additional help, some packages can only be built natively. RASS provides two solutions for this problem. Both solutions can be used together for a more distributed build.

* Remote build machines
  * Passing the `--build-machine` flag, you can have Nix log into a remote machine of the target architecture to perform the build. Run `rass --help` for more details. The only requirement of the remote machine is that you have ssh access to it and it has the Nix package manager installed. It does not need RASS installed.
* Setting up QEMU userspace emulation
  * If you are running on NixOS, the documentation for that is [here](https://nixos.wiki/wiki/NixOS_on_ARM#Compiling_through_binfmt_QEMU).
  * For a Debian derivative, see [Debian's documentation](https://wiki.debian.org/QemuUserEmulation).