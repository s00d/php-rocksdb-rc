set -ex
cd ../
docker build . -t module-builder --build-arg FROM_PHP=8.1
cd integration-test
docker run -it --rm -v ~/.cargo/registry:/root/.cargo/registry \
    -v ~/.cargo/git:/root/.cargo/git \
    -v $PWD/..:/code module-builder:latest bash -c 'cargo build --release'
cp ../target/release/lib*.so ./module.so
