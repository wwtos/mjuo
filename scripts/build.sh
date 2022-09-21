#!/bin/bash

# first, let's move into the main folder
if [ "${PWD##*/}" != "mjuo" ]; then
    cd ..
fi

working_directory="$PWD"

# make a build folder
mkdir -p build

# build the backend
cd vpo-backend
#cargo build --profile release-x86_64-pc-windows-gnu --target x86_64-pc-windows-gnu # windows
cargo build --profile release-x86_64-unknown-linux-gnu --target x86_64-unknown-linux-gnu # linux

# build the frontend
cd "$working_directory"
npm install
npx electron-packager . mjuo-frontend --overwrite --platform=linux --arch=x64 --icon=assets/icons/png/1024x1024.png --prune=true --out=../build
