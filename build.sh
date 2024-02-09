#!/bin/bash

ROOT=$(pwd)
VERSION="v0.1.0-alpha"

# part 0, make sure targets are present
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

# part 1, setup
mkdir -p build
rm -Rf build/*
mkdir -p build/linux
mkdir -p build/windows

# part 2, build the backend
cd "$ROOT/vpo-backend"

cargo build --release
cargo build --release --target x86_64-pc-windows-gnu

cp target/release/vpo-backend $ROOT/build/linux
cp target/x86_64-pc-windows-gnu/release/vpo-backend.exe $ROOT/build/windows

# part 3, build the frontend
cd $ROOT/vpo-frontend

npm run build

cp -R build $ROOT/build/linux/frontend
cp -R build $ROOT/build/windows/frontend

# part 4, package everything up
cd $ROOT/build/linux
tar -czvf "mjuo-linux-$VERSION.tar.gz" ./*
mv "mjuo-linux-$VERSION.tar.gz" ..

cd $ROOT/build/windows
zip -r "mjuo-windows-$VERSION.zip" .
mv "mjuo-windows-$VERSION.zip" ..