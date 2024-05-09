#!/bin/bash
## Install sui binary from latest mainnet release
echo $PWD
repo_url="https://api.github.com/repos/MystenLabs/sui/releases/latest"
latest_tag=$(curl -sSL "$repo_url" | jq -r '.tag_name')
ubuntu_artifact="sui-mainnet-${latest_tag#mainnet-}-ubuntu-x86_64.tgz"
browser_download_url=$(curl -sSL "$repo_url" | jq -r --arg ua "$ubuntu_artifact" '.assets[] | select(.name == $ua) | .browser_download_url')
wget -qO "$ubuntu_artifact" "$browser_download_url"

sudo tar -xzvf "$ubuntu_artifact" -C /usr/local/bin
rm "$ubuntu_artifact"
which sui