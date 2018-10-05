# This script takes care of testing your crate

set -ex

main() {
#    case "$TRAVIS_RUST_VERSION" in
#        nightly)
#            cargo +nightly fmt -- --check
#            ;;
#        *)
#            ;;
#    esac
#    cargo clippy -- -D warnings

    cargo build --target $TARGET
    cargo build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cargo test --target $TARGET
    cargo test --target $TARGET --release

    cargo run --target $TARGET --bin main assets/test.flv
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
