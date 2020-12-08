# Ansible

`ansible-playbook site.yml -i inventory/hosts.ini`

## Fresh install config

```
sudo hostnamectl set-hostname rasp-*

ssh-copy-id -i ./rasp.pub ubuntu@192.168.1.142
sudo vim /etc/ssh/sshd_config
sudo systemctl restart ssh
```

# KinD

Deploy a service and expose it to the host

```
kind create cluster --config kind-config.yml
kubectl create deployment hello-k8s  --image=geerlingguy/kube101:intro
kubectl create service nodeport hello-k8s --node-port=32000 --tcp=80:80
```

- [Using extraPortMappings in Windows](https://github.com/kubernetes-sigs/kind/issues/1523)
