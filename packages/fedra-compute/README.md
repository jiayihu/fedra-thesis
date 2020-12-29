# Ansible

`ansible-playbook site.yml -i inventory/hosts.ini`

## Alpine Linux fresh install

```
apk update
apk upgrade

# Enable root ssh
vi /etc/ssh/sshd_config
rc-service sshd restart

# Add wpa_supplicant service
rc-update add wpa_supplicant default
# https://gitlab.alpinelinux.org/alpine/aports/-/issues/8025
vi /etc/init.d/networking # need wpa_supplicant
rc-update -u

# Enable only SSH keys
ssh-copy-id root@rasp-4
vi /etc/ssh/sshd_config # PasswordAuthentication no
rc-service sshd restart

# Install python
apk add python3

vi /etc/rc.conf # rc_logger="YES"

reboot
```
