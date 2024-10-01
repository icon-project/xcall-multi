#!/bin/bash
## Install sui binary from latest mainnet release
echo $PWD
repo_url="https://api.github.com/repos/MystenLabs/sui/releases"
latest_tag=$(curl -sSL "$repo_url" | jq -r '.[] | select(.tag_name | contains("mainnet")) | .tag_name' | head -n 1)
ubuntu_artifact="sui-mainnet-${latest_tag#mainnet-}-ubuntu-x86_64.tgz"
browser_download_url=$(curl -sSL "$repo_url" | jq -r --arg ua "$ubuntu_artifact" '.[] | select(.tag_name == "'$latest_tag'") | .assets[] | select(.name == $ua) | .browser_download_url')
wget -qO "$ubuntu_artifact" "$browser_download_url"
sudo tar -xzvf "$ubuntu_artifact" -C /usr/local/bin
rm "$ubuntu_artifact"
which sui
