#!/bin/bash

# Install neovim
curl -OL https://github.com/neovim/neovim/releases/download/stable/nvim-linux64.tar.gz
tar -xzvf nvim-linux64.tar.gz
echo "export PATH=\"$HOME/nvim-linux64/bin:\$PATH\"" >> ~/.bashrc

# Configure AstroNvim with my config
git clone --depth 1 https://github.com/AstroNvim/AstroNvim ~/.config/nvim
git clone https://github.com/vuongDang/astronvim-config.git ~/.config/nvim/lua/user

# Configure shell
echo 'export PS1="\n\[$(tput setaf 215)\]\t \[\033[32m\]\[$(tput setaf 39)\]\u\[$(tput setaf 81)\]@\[$(tput setaf 77)\]\h \[$(tput setaf 226)\]\w\[\033[00m\] $ "' >> ~/.bashrc
echo "alias ls='ls --color=auto'" >> ~/.bashrc

. ~/.bashrc



