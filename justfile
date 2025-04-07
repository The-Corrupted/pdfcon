build:
    cargo build

release:
    cargo build --release

install:
    sudo mv target/release/pdfcon /usr/bin

fix:
    cargo fix
