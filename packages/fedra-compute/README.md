# Ansible

`ansible-playbook site.yml -i inventory/hosts.ini`

## Fresh install config

```
sudo hostnamectl set-hostname rasp-*

ssh-copy-id -i ./rasp.pub ubuntu@192.168.1.142
sudo vim /etc/ssh/sshd_config
sudo systemctl restart ssh
```
