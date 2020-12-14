# Ansible

`ansible-playbook site.yml -i inventory/hosts.ini`

## Alpine Linux fresh install

```
apk update
apk upgrade

# Enable root ssh
vi /etc/ssh/sshd_config
rc-service sshd restart

# Add /etc/init.d to lbu
lbu add /etc/init.d
lbu add /root/.ssh
vi /etc/lbu/lbu.conf # BACKUP_LIMIT=5

# Add wpa_supplicant service
rc-update add wpa_supplicant default
# https://gitlab.alpinelinux.org/alpine/aports/-/issues/8025
vi /etc/init.d/networking # need wpa_supplicant
rc-update -u
lbu ci -d

# Enable only SSH keys
ssh-copy-id root@rasp-4
vi /etc/ssh/sshd_config # PasswordAuthentication no
rc-service sshd restart

# Install python
apk add python3

# https://wiki.alpinelinux.org/wiki/Raspberry_Pi#Persistent_storage
mount /media/mmcblk0p1 -o rw,remount
sed -i 's/vfat\ ro,/vfat\ rw,/' /etc/fstab
dd if=/dev/zero of=/media/mmcblk0p1/persist.img bs=1024 count=0 seek=1048576
apk add e2fsprogs
mkfs.ext4 /media/mmcblk0p1/persist.img
echo "/media/mmcblk0p1/persist.img /media/persist ext4 rw,relatime,errors=remount-ro 0 0" >> /etc/fstab
mkdir /media/persist 
mount -a

# Persist /usr
mkdir /media/persist/usr
mkdir /media/persist/.work
echo "overlay /usr overlay lowerdir=/usr,upperdir=/media/persist/usr,workdir=/media/persist/.work 0 0" >> /etc/fstab 
mount -a

# Persist /var/lib/rancher
mkdir /media/persist/rancher 
mkdir /media/persist/.work-rancher
mkdir /var/lib/rancher
echo "overlay /var/lib/rancher overlay lowerdir=/var/lib/rancher,upperdir=/media/persist/rancher,workdir=/media/persist/.work-rancher 0 0" >> /etc/fstab 
mount -a

# Persist /root
mkdir /media/persist/root 
mkdir /media/persist/.work-root
echo "overlay /root overlay lowerdir=/root,upperdir=/media/persist/root,workdir=/media/persist/.work-root 0 0" >> /etc/fstab 
mount -a

cat /etc/fstab

vi /etc/rc.conf # rc_logger="YES"
lbu ci -d

reboot
```
