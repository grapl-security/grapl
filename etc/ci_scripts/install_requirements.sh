
# wget https://github.com/dnephin/dobi/releases/download/v0.13.0/dobi-linux
# chmod +x dobi-linux

# update docker
docker --version
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu  $(lsb_release -cs)  stable"
sudo apt-get update
sudo apt install docker-ce docker-ce-cli containerd.io
docker --version